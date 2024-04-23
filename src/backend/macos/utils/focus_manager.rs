use objc2::rc::Id;

use objc2_app_kit::{NSApplication, NSWindow};
use objc2_foundation::MainThreadMarker;

pub struct FocusManager {
    key_window: Option<Id<NSWindow>>,
}

impl FocusManager {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let app = NSApplication::sharedApplication(mtm);
        let key_window = app.keyWindow();

        Self { key_window }
    }
}

impl Drop for FocusManager {
    fn drop(&mut self) {
        if let Some(win) = &self.key_window {
            win.makeKeyAndOrderFront(None);
        }
    }
}
