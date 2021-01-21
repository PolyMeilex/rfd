use std::ffi::CString;
use std::ptr;

use crate::dialog::{MessageButtons, MessageDialog, MessageLevel};
use crate::MessageDialogImpl;

pub struct GtkMessageDialog {
    ptr: *mut gtk_sys::GtkDialog,
}

impl GtkMessageDialog {
    pub fn new(opt: MessageDialog) -> Self {
        super::gtk_init_check();

        let s: &str = &opt.text;
        let text = CString::new(s).unwrap();

        let level = match opt.level {
            MessageLevel::Info => gtk_sys::GTK_MESSAGE_INFO,
            MessageLevel::Warning => gtk_sys::GTK_MESSAGE_WARNING,
            MessageLevel::Error => gtk_sys::GTK_MESSAGE_ERROR,
        };

        let buttons = match opt.buttons {
            MessageButtons::Ok => gtk_sys::GTK_BUTTONS_OK,
            MessageButtons::OkCancle => gtk_sys::GTK_BUTTONS_OK_CANCEL,
            MessageButtons::YesNo => gtk_sys::GTK_BUTTONS_YES_NO,
        };

        let ptr = unsafe {
            gtk_sys::gtk_message_dialog_new(
                ptr::null_mut(),
                gtk_sys::GTK_DIALOG_MODAL,
                level,
                buttons,
                b"%s\0".as_ptr() as *mut _,
                text.as_ptr(),
            ) as *mut gtk_sys::GtkDialog
        };

        Self { ptr }
    }

    pub fn run(self) -> bool {
        let res = unsafe { gtk_sys::gtk_dialog_run(self.ptr) };

        unsafe {
            gtk_sys::gtk_widget_destroy(self.ptr as *mut _);
            super::wait_for_cleanup();
        }

        res == gtk_sys::GTK_RESPONSE_OK || res == gtk_sys::GTK_RESPONSE_YES
    }
}

impl MessageDialogImpl for MessageDialog {
    fn show(self) {
        let dialog = GtkMessageDialog::new(self);
        let res = dialog.run();

        println!("{:?}", res);
    }
}
