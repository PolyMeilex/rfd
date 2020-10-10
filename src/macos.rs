use crate::DialogParams;
use std::path::PathBuf;

// use block::ConcreteBlock;

use objc::runtime::{Class, Object};
use objc::{class, msg_send, sel, sel_impl};
// use objc_id::ShareId;

use objc::runtime;

pub use objc::runtime::{BOOL, NO, YES};

// pub type Class = *mut runtime::Class;
// #[allow(non_camel_case_types)]
// pub type id = *mut runtime::Object;
// pub type SEL = runtime::Sel;

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

pub unsafe fn NSApp() -> *mut Object {
    msg_send![class!(NSApplication), sharedApplication]
}

pub unsafe fn key_window() -> *mut Object {
    let sa: *mut Object = msg_send![class!(NSApplication), sharedApplication];
    msg_send![sa, keyWindow]
}

fn retain_count(obj: *mut Object) -> usize {
    unsafe { msg_send![obj, retainCount] }
}

fn panel() -> *mut Object {
    unsafe {
        let cls = class!(NSOpenPanel);
        let panel: *mut Object = msg_send![cls, openPanel];
        // println!("{}", retain_count(&*panel));
        // ShareId::from_ptr(panel)
        panel
    }
}

extern "C" {
    fn CGShieldingWindowLevel() -> i32;
}

pub fn open(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        // NSAutoreleasePool *pool = [[NSAutoreleasePool alloc] init];
        let pool = NSAutoreleasePool::new(nil);

        // NSWindow *keyWindow = [[NSApplication sharedApplication] keyWindow];
        // let _app = key_window();

        // NSOpenPanel *dialog = [NSOpenPanel openPanel];
        // let cls = class!(NSOpenPanel);
        // let panel: id = msg_send![cls, openPanel];

        let p = {
            let panel = panel();

            let level = CGShieldingWindowLevel();
            let _: () = msg_send![panel, setLevel: level];

            let _: () = msg_send![panel, setCanChooseDirectories: YES];
            let _: () = msg_send![panel, setCanChooseFiles: YES];

            // println!("{}", retain_count(panel));

            let _: () = msg_send![panel, runModal];

            // println!("{}", retain_count(panel));

            panel as *mut Object
        };

        println!("{}", retain_count(p));

        pool.release();

        // pool.autorelease();
        // let id = ShareId::from_ptr(x)
    }

    None
}
