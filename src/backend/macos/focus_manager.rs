use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};
use objc_id::Id;

use super::utils::{nil, INSApplication, NSApplication};

pub struct FocusManager {
    key_window: Option<Id<Object>>,
}

impl FocusManager {
    pub fn new() -> Self {
        let app = NSApplication::shared_application();
        let key_window = app.key_window();

        let key_window = if key_window.is_null() {
            None
        } else {
            unsafe { Some(Id::from_ptr(key_window)) }
        };

        Self { key_window }
    }
}

impl Drop for FocusManager {
    fn drop(&mut self) {
        if let Some(win) = &self.key_window {
            let _: () = unsafe { msg_send![*win, makeKeyAndOrderFront: nil] };
        }
    }
}
