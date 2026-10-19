use std::cell::{Cell, RefCell};
use std::ptr;
use std::rc::Rc;
use std::sync::{mpsc, Once};

use glib::prelude::*;
use glib::translate::from_glib_none;

use super::AsGtkDialog;

/// Makes sure something is running the GTK event loop
fn ensure_gtk_thread() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // GTK3 has no `gtk_is_initialized`, but `gtk_init` opens the default display
        if unsafe { gdk_sys::gdk_display_get_default() }.is_null() {
            let (tx, rx) = mpsc::channel();

            std::thread::spawn(move || {
                let initialized =
                    unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) } != 0;
                tx.send(initialized).ok();
                if initialized {
                    unsafe { gtk_sys::gtk_main() };
                }
            });

            let initialized = rx.recv().expect("GTK3 thread exited before initializing");
            assert!(initialized, "failed to initialize GTK3");
        }
    });
}

/// Runs the function on GTK thread
fn run_on_main<F: FnOnce() + Send + 'static>(f: F) {
    ensure_gtk_thread();
    glib::idle_add_once(f);
}

/// Builds and shows a dialog, must be called on GTK thread
fn show_dialog_local<D, R, B, F, Done>(build: B, cb: F, done: Done)
where
    D: AsGtkDialog + 'static,
    R: 'static,
    B: FnOnce() -> D,
    F: FnOnce(&D, i32) -> R + 'static,
    Done: FnOnce(R) + 'static,
{
    let dialog = build();
    let object: glib::Object =
        unsafe { from_glib_none(dialog.gtk_dialog_ptr() as *mut gobject_sys::GObject) };

    unsafe { dialog.show() };

    let state = RefCell::new(Some((dialog, cb, done)));
    object.connect_local("response", false, move |args| {
        let object = args[0]
            .get::<glib::Object>()
            .expect("signal emitter should be an object");
        let res = args[1].get::<i32>().expect("response id should be an int");

        if let Some((dialog, cb, done)) = state.borrow_mut().take() {
            let res = cb(&dialog, res);
            // Destroy the dialog outside of its own signal
            glib::idle_add_local_once(move || {
                drop(dialog);
                // GTK may have already destroyed the dialog (eg. on delete-event), so keep it alive until now
                drop(object);
                done(res);
            });
        }

        None
    });
}

/// Shows a dialog on GTK thread, and returns `cb` result once it responds
pub(super) fn spawn_dialog<D, R, B, F>(build: B, cb: F) -> crate::backend::DialogFutureType<R>
where
    D: AsGtkDialog + 'static,
    R: Send + 'static,
    B: FnOnce() -> D + Send + 'static,
    F: FnOnce(&D, i32) -> R + Send + 'static,
{
    let (tx, rx) = crate::oneshot::channel();
    run_on_main(move || {
        show_dialog_local(build, cb, move |res| {
            tx.send(res).ok();
        })
    });
    Box::pin(async move { rx.await.expect("GTK3 dialog was dropped") })
}

/// Shows a dialog on GTK thread, and blocks until it responds
pub(super) fn run_dialog<D, R, B, F>(build: B, cb: F) -> R
where
    D: AsGtkDialog + 'static,
    R: Send + 'static,
    B: FnOnce() -> D + Send + 'static,
    F: FnOnce(&D, i32) -> R + Send + 'static,
{
    if glib::MainContext::default().is_owner() {
        // We are already on GTK thread, so we run a nested main loop until the dialog responds
        let main_loop = glib::MainLoop::new(None, false);
        let result = Rc::new(Cell::new(None));

        show_dialog_local(build, cb, {
            let main_loop = main_loop.clone();
            let result = result.clone();
            move |res| {
                result.set(Some(res));
                main_loop.quit();
            }
        });

        main_loop.run();
        return result.take().expect("GTK3 dialog was dropped");
    }

    let (tx, rx) = mpsc::channel();
    run_on_main(move || {
        show_dialog_local(build, cb, move |res| {
            tx.send(res).ok();
        })
    });
    rx.recv().expect("GTK3 dialog was dropped")
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
