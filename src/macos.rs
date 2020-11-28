use crate::DialogParams;
use std::path::PathBuf;

use objc::runtime::{Class, Object};
use objc::{class, msg_send, sel, sel_impl};

use objc::runtime;

pub use objc::runtime::{BOOL, NO, YES};

#[allow(non_upper_case_globals)]
pub const nil: *mut Object = 0 as *mut Object;
#[allow(non_upper_case_globals)]
pub const Nil: *mut Class = 0 as *mut Class;

pub trait NSAutoreleasePool: Sized {
    unsafe fn new(_: Self) -> *mut Object {
        msg_send![class!(NSAutoreleasePool), new]
    }

    unsafe fn release(self);
}

impl NSAutoreleasePool for *mut Object {
    unsafe fn release(self) {
        msg_send![self, release]
    }
}

pub unsafe fn shared_application() -> *mut Object {
    msg_send![class!(NSApplication), sharedApplication]
}

pub unsafe fn key_window() -> *mut Object {
    let shared_app = shared_application();
    msg_send![shared_app, keyWindow]
}

fn retain_count(obj: *mut Object) -> usize {
    unsafe { msg_send![obj, retainCount] }
}

fn panel() -> *mut Object {
    unsafe {
        let cls = class!(NSOpenPanel);
        let panel: *mut Object = msg_send![cls, openPanel];
        panel
    }
}

extern "C" {
    fn CGShieldingWindowLevel() -> i32;
}

#[repr(i32)]
#[derive(Debug, PartialEq)]
enum ApplicationActivationPolicy {
    Regular = 0,
    Accessory = 1,
    Prohibited = 2,
    Error = -1,
}

pub fn open_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let shared_app = shared_application();
        // NSWindow *keyWindow = [[NSApplication sharedApplication] keyWindow];
        let key_window = key_window();

        let prev_policy = {
            let sa: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let pol: i32 = msg_send![sa, activationPolicy];

            if pol == ApplicationActivationPolicy::Prohibited as i32 {
                let new_pol = ApplicationActivationPolicy::Accessory as i32;
                let _: () = msg_send![sa, setActivationPolicy: new_pol];
            }
            pol
        };

        let p = {
            let panel = panel();

            let level = CGShieldingWindowLevel();
            let _: () = msg_send![panel, setLevel: level];

            let _: () = msg_send![panel, setCanChooseDirectories: YES];
            let _: () = msg_send![panel, setCanChooseFiles: YES];

            let _: () = msg_send![panel, runModal];

            panel as *mut Object
        };

        let _: () = msg_send![key_window, makeKeyAndOrderFront: nil];
        let _: () = msg_send![shared_app, setActivationPolicy: prev_policy];

        pool.release();
    }

    None
}
