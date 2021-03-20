use std::ops::DerefMut;

use crate::backend::DialogFutureType;
use crate::dialog::{MessageButtons, MessageDialog, MessageLevel};

use super::modal_future::ModalFuture;
use super::utils::{nil, run_on_main};
use super::AsModal;

use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};
use objc_foundation::{INSString, NSString};

use objc_id::Id;

#[repr(i64)]
#[derive(Debug, PartialEq)]
enum NSAlertStyle {
    Warning = 0,
    Informational = 1,
    Critical = 2,
}

#[repr(i64)]
#[derive(Debug, PartialEq)]
enum NSAlertReturn {
    FirstButton = 1000,
    // SecondButton = 1001,
    // ThirdButton = 1002,
}

pub struct NSAlert {
    alert: Id<Object>,
    key_window: *mut Object,
}

impl NSAlert {
    pub fn new(opt: MessageDialog) -> Self {
        let alert: *mut Object = unsafe { msg_send![class!(NSAlert), new] };

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
                let label = NSString::from_str("OK");
                let _: () = msg_send![alert, addButtonWithTitle: label];
            },
            MessageButtons::OkCancle => unsafe {
                let label = NSString::from_str("OK");
                let _: () = msg_send![alert, addButtonWithTitle: label];
                let label = NSString::from_str("Cancel");
                let _: () = msg_send![alert, addButtonWithTitle: label];
            },
            MessageButtons::YesNo => unsafe {
                let label = NSString::from_str("Yes");
                let _: () = msg_send![alert, addButtonWithTitle: label];
                let label = NSString::from_str("No");
                let _: () = msg_send![alert, addButtonWithTitle: label];
            },
        }

        unsafe {
            let text = NSString::from_str(&opt.title);
            let _: () = msg_send![alert, setMessageText: text];
            let text = NSString::from_str(&opt.description);
            let _: () = msg_send![alert, setInformativeText: text];
        }

        let key_window = unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            msg_send![app, keyWindow]
        };

        Self {
            alert: unsafe { Id::from_retained_ptr(alert) },
            key_window,
        }
    }

    pub fn run(self) -> bool {
        let ret: i64 = unsafe { msg_send![self.alert, runModal] };
        ret == NSAlertReturn::FirstButton as i64
    }
}

impl AsModal for NSAlert {
    fn modal_ptr(&mut self) -> *mut Object {
        self.alert.deref_mut()
    }
}

impl Drop for NSAlert {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.key_window, makeKeyAndOrderFront: nil] };
    }
}

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> bool {
        objc::rc::autoreleasepool(move || run_on_main(move || NSAlert::new(self).run()))
    }
}

use crate::backend::AsyncMessageDialogImpl;

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<bool> {
        let future = ModalFuture::new(
            move || NSAlert::new(self),
            |_, res_id| res_id == NSAlertReturn::FirstButton as i64,
        );
        Box::pin(future)
    }
}
