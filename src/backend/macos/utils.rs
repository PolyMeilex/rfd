use std::path::PathBuf;

use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

use objc_foundation::{object_struct, INSObject, INSString, NSString};
use objc_id::Id;

#[allow(non_upper_case_globals)]
pub const nil: *mut Object = 0 as *mut _;

pub fn is_main_thread() -> bool {
    unsafe { msg_send![class!(NSThread), isMainThread] }
}

pub fn activate_cocoa_multithreading() {
    unsafe {
        let thread: *mut Object = msg_send![class!(NSThread), new];
        let _: () = msg_send![thread, start];
    }
}

pub struct NSApplication(pub *mut Object);

impl NSApplication {
    pub fn shared_application() -> Self {
        Self(unsafe { msg_send![class!(NSApplication), sharedApplication] })
    }

    pub fn is_running(&self) -> bool {
        unsafe { msg_send![self.0, isRunning] }
    }

    pub fn key_window(&self) -> *mut Object {
        unsafe { msg_send![self.0, keyWindow] }
    }
}

pub fn run_on_main<R: Send, F: FnOnce() -> R + Send>(run: F) -> R {
    if is_main_thread() {
        run()
    } else {
        let app = NSApplication::shared_application();
        if app.is_running() {
            let main = dispatch::Queue::main();
            main.exec_sync(run)
        } else {
            panic!("You are running RFD in NonWindowed environment, it is impossible to spawn dialog from thread different than main in this env.");
        }
    }
}

pub trait INSURL: INSObject {
    fn file_url_with_path(s: &str, is_dir: bool) -> Id<Self> {
        let s = NSString::from_str(s);
        let ptr = unsafe { msg_send![class!(NSURL), fileURLWithPath: s isDirectory:is_dir] };
        unsafe { Id::from_retained_ptr(ptr) }
    }

    fn to_path_buf(&self) -> PathBuf {
        let s = unsafe { msg_send![self, path] };
        let s: Id<NSString> = unsafe { Id::from_ptr(s) };
        s.as_str().into()
    }
}

object_struct!(NSURL);
impl INSURL for NSURL {}
