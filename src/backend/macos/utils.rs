mod focus_manager;
mod policy_manager;
mod user_alert;

pub use focus_manager::FocusManager;
pub use policy_manager::PolicyManager;
pub use user_alert::{async_pop_dialog, sync_pop_dialog};

use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

use dispatch2::run_on_main as dispatch_on_main;
use objc2::rc::{autoreleasepool, Retained};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSView, NSWindow};
use objc2_foundation::NSThread;
use raw_window_handle::RawWindowHandle;

/// Notify Cocoa of multithreaded use by starting a short-lived `NSThread`.
/// Application work runs separately.
pub fn activate_cocoa_multithreading() {
    let thread = NSThread::new();
    unsafe { thread.start() };
}

/// Run a closure on the OS main thread.
///
/// Background callers wait until the closure finishes, so main must service its
/// dispatch queue. This function does not start or stop native event processing.
///
/// # Panics
///
/// A Rust panic from the closure is resumed on the calling thread.
/// Native completion callbacks invoked later are outside this boundary.
pub fn run_on_main<R: Send, F: FnOnce(MainThreadMarker) -> R + Send>(run: F) -> R {
    let result = dispatch_on_main(move |main| {
        // Catch Rust panics before they unwind through dispatch's C boundary.
        // The pool drains temporary native objects on main.
        catch_unwind(AssertUnwindSafe(|| autoreleasepool(|_| run(main))))
    });
    match result {
        Ok(value) => value,
        Err(panic) => resume_unwind(panic),
    }
}

pub fn window_from_raw_window_handle(
    h: &RawWindowHandle,
    _main: MainThreadMarker,
) -> Retained<NSWindow> {
    match h {
        RawWindowHandle::AppKit(h) => {
            let view = h.ns_view.as_ptr() as *mut NSView;
            let view = unsafe { Retained::retain(view).unwrap() };
            view.window().expect("NSView to be inside a NSWindow")
        }
        _ => unreachable!("unsupported window handle, expected: MacOS"),
    }
}
