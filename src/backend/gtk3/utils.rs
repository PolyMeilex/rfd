use glib::prelude::*;
use glib::translate::FromGlibPtrContainer;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::spawn;

static GTK_THREAD: OnceLock<GtkGlobalThread> = OnceLock::new();

/// GTK functions are not thread-safe, and must all be called from the thread that initialized GTK. To ensure this, we
/// spawn one thread the first time a GTK dialog is opened and keep it open for the entire lifetime of the application,
/// as GTK cannot be de-initialized or re-initialized on another thread. You're stuck on the thread on which you first
/// initialize GTK.
pub struct GtkGlobalThread {
    running: Arc<AtomicBool>,
}

impl GtkGlobalThread {
    /// Return the global, lazily-initialized instance of the global GTK thread.
    pub(super) fn instance() -> &'static Self {
        GTK_THREAD.get_or_init(|| Self::new())
    }

    fn new() -> Self {
        // When the GtkGlobalThread is eventually dropped, we will set `running` to false and wake up the loop so
        // gtk_main_iteration unblocks and we exit the thread on the next iteration.
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = Arc::clone(&running);

        spawn(move || {
            let initialized =
                unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1 };
            if !initialized {
                return;
            }

            loop {
                if !thread_running.load(Ordering::Acquire) {
                    break;
                }

                unsafe {
                    gtk_sys::gtk_main_iteration();
                }
            }
        });

        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Run a function on the GTK thread, blocking on the result which is then passed back.
    pub(super) fn run_blocking<
        T: Send + Clone + std::fmt::Debug + 'static,
        F: FnOnce() -> T + Send + 'static,
    >(
        &self,
        cb: F,
    ) -> T {
        let data: Arc<(Mutex<Option<T>>, _)> = Arc::new((Mutex::new(None), Condvar::new()));
        let thread_data = Arc::clone(&data);
        glib::idle_add_once(move || {
            let res = cb();

            // pass the result back to the main thread
            let (lock, cvar) = &*thread_data;
            *lock.lock().unwrap() = Some(res);
            cvar.notify_all();
        });

        // wait for GTK thread to execute the callback and place the result into `data`
        let lock_res = data
            .1
            .wait_while(data.0.lock().unwrap(), |res| res.is_none())
            .unwrap();
        lock_res.as_ref().unwrap().clone()
    }

    /// Launch a function on the GTK thread without blocking.
    pub(super) fn run<F: FnOnce() + Send + 'static>(&self, cb: F) {
        glib::idle_add_once(cb);
    }
}

impl Drop for GtkGlobalThread {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        glib::MainContext::default().wakeup();
    }
}

pub(super) unsafe fn find_gtk_window(
    parent: &raw_window_handle::RawWindowHandle,
) -> Option<glib::Object> {
    let toplevels: Vec<glib::Object> =
        FromGlibPtrContainer::from_glib_container(gtk_sys::gtk_window_list_toplevels());

    type GdkX11WindowGetXidFn =
        unsafe extern "C" fn(*mut std::ffi::c_void) -> std::os::raw::c_ulong;
    type GdkWaylandWindowGetWlSurfaceFn =
        unsafe extern "C" fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void;

    let gdk_x11_window_get_xid: Option<GdkX11WindowGetXidFn> = std::mem::transmute(libc::dlsym(
        libc::RTLD_DEFAULT,
        b"gdk_x11_window_get_xid\0".as_ptr() as *const _,
    ));
    let gdk_wayland_window_get_wl_surface: Option<GdkWaylandWindowGetWlSurfaceFn> =
        std::mem::transmute(libc::dlsym(
            libc::RTLD_DEFAULT,
            b"gdk_wayland_window_get_wl_surface\0".as_ptr() as *const _,
        ));

    toplevels.into_iter().find(|window| {
        let window = window.as_ptr() as *mut gtk_sys::GtkWidget;
        if gtk_sys::gtk_widget_get_realized(window) == 0 {
            return false;
        }

        let gdk_window = gtk_sys::gtk_widget_get_window(window) as *mut std::ffi::c_void;
        if gdk_window.is_null() {
            return false;
        }

        match parent {
            raw_window_handle::RawWindowHandle::Xlib(h) => {
                gdk_x11_window_get_xid.is_some_and(|get_xid| get_xid(gdk_window) == h.window)
            }
            raw_window_handle::RawWindowHandle::Xcb(h) => {
                gdk_x11_window_get_xid.is_some_and(|get_xid| {
                    get_xid(gdk_window) == h.window.get() as std::os::raw::c_ulong
                })
            }
            raw_window_handle::RawWindowHandle::Wayland(h) => gdk_wayland_window_get_wl_surface
                .is_some_and(|get_surface| get_surface(gdk_window) == h.surface.as_ptr()),
            _ => false,
        }
    })
}
