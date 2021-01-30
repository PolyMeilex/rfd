use cocoa_foundation::base::id;
use objc::{class, msg_send, sel, sel_impl};

pub fn is_main_thread() -> bool {
    unsafe { msg_send![class!(NSThread), isMainThread] }
}

pub fn shared_application() -> id {
    unsafe { msg_send![class!(NSApplication), sharedApplication] }
}

pub fn activate_cocoa_multithreading() {
    unsafe {
        let thread: id = msg_send![class!(NSThread), new];
        let _: () = msg_send![thread, start];
    }
}
