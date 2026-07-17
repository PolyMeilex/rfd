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
        let mut cb = Some(cb);
        unsafe {
            connect_idle(move || {
                // connect_idle takes a FnMut; convert our FnOnce into that by ensuring we only call it once
                let res = cb.take().expect("Callback should only be called once")();

                // pass the result back to the main thread
                let (lock, cvar) = &*thread_data;
                *lock.lock().unwrap() = Some(res);
                cvar.notify_all();

                glib_sys::GFALSE
            });
        };

        // wait for GTK thread to execute the callback and place the result into `data`
        let lock_res = data
            .1
            .wait_while(data.0.lock().unwrap(), |res| res.is_none())
            .unwrap();
        lock_res.as_ref().unwrap().clone()
    }

    /// Launch a function on the GTK thread without blocking.
    pub(super) fn run<F: FnOnce() + Send + 'static>(&self, cb: F) {
        let mut cb = Some(cb);
        unsafe {
            connect_idle(move || {
                cb.take().expect("Callback should only be called once")();
                glib_sys::GFALSE
            });
        };
    }
}

impl Drop for GtkGlobalThread {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        unsafe { glib_sys::g_main_context_wakeup(std::ptr::null_mut()) };
    }
}

unsafe fn connect_idle<F: FnMut() -> glib_sys::gboolean + Send + 'static>(f: F) {
    unsafe extern "C" fn response_trampoline<F: FnMut() -> glib_sys::gboolean + Send + 'static>(
        f: glib_sys::gpointer,
    ) -> glib_sys::gboolean {
        let f: &mut F = &mut *(f as *mut F);

        f()
    }
    let f_box: Box<F> = Box::new(f);

    unsafe extern "C" fn destroy_closure<F>(ptr: *mut std::ffi::c_void) {
        // destroy
        let _ = Box::<F>::from_raw(ptr as *mut _);
    }

    glib_sys::g_idle_add_full(
        glib_sys::G_PRIORITY_DEFAULT_IDLE,
        Some(response_trampoline::<F>),
        Box::into_raw(f_box) as glib_sys::gpointer,
        Some(destroy_closure::<F>),
    );
}

pub(super) unsafe fn find_gtk_window(
    parent: &raw_window_handle::RawWindowHandle,
) -> *mut gtk_sys::GtkWindow {
    let toplevels = gtk_sys::gtk_window_list_toplevels();
    let mut current = toplevels;
    let mut found_window = std::ptr::null_mut();

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

    while !current.is_null() {
        let window = (*current).data as *mut gtk_sys::GtkWindow;
        if gtk_sys::gtk_widget_get_realized(window as _) != 0 {
            let gdk_window = gtk_sys::gtk_widget_get_window(window as _) as *mut std::ffi::c_void;
            if !gdk_window.is_null() {
                match parent {
                    raw_window_handle::RawWindowHandle::Xlib(h) => {
                        if let Some(get_xid) = gdk_x11_window_get_xid {
                            if get_xid(gdk_window) == h.window {
                                found_window = window;
                                break;
                            }
                        }
                    }
                    raw_window_handle::RawWindowHandle::Xcb(h) => {
                        if let Some(get_xid) = gdk_x11_window_get_xid {
                            if get_xid(gdk_window) == h.window.get() as std::os::raw::c_ulong {
                                found_window = window;
                                break;
                            }
                        }
                    }
                    raw_window_handle::RawWindowHandle::Wayland(h) => {
                        if let Some(get_surface) = gdk_wayland_window_get_wl_surface {
                            if get_surface(gdk_window) == h.surface.as_ptr() {
                                found_window = window;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        current = (*current).next;
    }

    glib_sys::g_list_free(toplevels);

    found_window
}
