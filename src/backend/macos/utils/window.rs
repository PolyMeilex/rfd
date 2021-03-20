use objc::{msg_send, sel, sel_impl};

use super::nil;
use objc_foundation::{object_struct, INSObject};

pub trait INSWindow: INSObject {
    fn make_key_and_order_front(&self) {
        let _: () = unsafe { msg_send![self, makeKeyAndOrderFront: nil] };
    }
}

object_struct!(NSWindow);
impl INSWindow for NSWindow {}
