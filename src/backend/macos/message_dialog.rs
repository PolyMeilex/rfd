use crate::dialog::{MessageButtons, MessageDialog, MessageLevel};

use cocoa_foundation::base::{id, nil};
use cocoa_foundation::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};

#[repr(i64)]
#[derive(Debug, PartialEq)]
enum NSAlertStyle {
    Warning = 0,
    Informational = 1,
    Critical = 2,
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    static NSAlertFirstButtonReturn: i64;
}

pub struct NSAlert {
    ptr: id,
    key_window: id,
}

impl NSAlert {
    pub fn new(opt: MessageDialog) -> Self {
        let alert: id = unsafe { msg_send![class!(NSAlert), new] };

        let level = match opt.level {
            MessageLevel::Info => NSAlertStyle::Informational,
            MessageLevel::Warning => NSAlertStyle::Warning,
            MessageLevel::Error => NSAlertStyle::Critical,
        };

        unsafe {
            let _: () = msg_send![alert, setAlertStyle: level as i64];
        }

        match opt.buttons {
            MessageButtons::Ok => unsafe {
                let label = NSString::alloc(nil).init_str("OK");
                let _: () = msg_send![alert, addButtonWithTitle: label];
            },
            MessageButtons::OkCancle => unsafe {
                let label = NSString::alloc(nil).init_str("OK");
                let _: () = msg_send![alert, addButtonWithTitle: label];
                let label = NSString::alloc(nil).init_str("Cancel");
                let _: () = msg_send![alert, addButtonWithTitle: label];
            },
            MessageButtons::YesNo => unsafe {
                let label = NSString::alloc(nil).init_str("Yes");
                let _: () = msg_send![alert, addButtonWithTitle: label];
                let label = NSString::alloc(nil).init_str("No");
                let _: () = msg_send![alert, addButtonWithTitle: label];
            },
        }

        unsafe {
            let text = NSString::alloc(nil).init_str(&opt.text);
            let _: () = msg_send![alert, setMessageText: text];
        }

        let key_window = unsafe {
            let app: id = msg_send![class!(NSApplication), sharedApplication];
            msg_send![app, keyWindow]
        };

        Self {
            ptr: alert,
            key_window,
        }
    }

    pub fn run(self) -> bool {
        let ret: i64 = unsafe { msg_send![self.ptr, runModal] };

        unsafe { ret == NSAlertFirstButtonReturn }
    }
}

impl Drop for NSAlert {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.key_window, makeKeyAndOrderFront: nil] };
    }
}

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) {
        let dialog = NSAlert::new(self);
        let res = dialog.run();

        println!("{:?}", res);
    }
}
