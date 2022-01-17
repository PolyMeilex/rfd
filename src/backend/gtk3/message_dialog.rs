use std::ffi::CString;
use std::ptr;

use super::gtk_future::GtkDialogFuture;
use super::utils::wait_for_cleanup;
use super::AsGtkDialog;

use crate::message_dialog::{MessageButtons, MessageDialog, MessageLevel};

pub struct GtkMessageDialog {
    ptr: *mut gtk_sys::GtkDialog,
}

impl GtkMessageDialog {
    pub fn new(opt: MessageDialog) -> Self {
        super::utils::gtk_init_check();

        let level = match opt.level {
            MessageLevel::Info => gtk_sys::GTK_MESSAGE_INFO,
            MessageLevel::Warning => gtk_sys::GTK_MESSAGE_WARNING,
            MessageLevel::Error => gtk_sys::GTK_MESSAGE_ERROR,
        };

        let buttons = match opt.buttons {
            MessageButtons::Ok => gtk_sys::GTK_BUTTONS_OK,
            MessageButtons::OkCancel => gtk_sys::GTK_BUTTONS_OK_CANCEL,
            MessageButtons::YesNo => gtk_sys::GTK_BUTTONS_YES_NO,
        };

        let s: &str = &opt.title;
        let title = CString::new(s).unwrap();
        let s: &str = &opt.description;
        let description = CString::new(s).unwrap();

        let ptr = unsafe {
            gtk_sys::gtk_message_dialog_new(
                ptr::null_mut(),
                gtk_sys::GTK_DIALOG_MODAL,
                level,
                buttons,
                b"%s\0".as_ptr() as *mut _,
                title.as_ptr(),
            ) as *mut gtk_sys::GtkDialog
        };

        unsafe {
            gtk_sys::gtk_message_dialog_format_secondary_text(ptr as *mut _, description.as_ptr());
        }

        Self { ptr }
    }

    pub fn run(self) -> bool {
        let res = unsafe { gtk_sys::gtk_dialog_run(self.ptr) };

        res == gtk_sys::GTK_RESPONSE_OK || res == gtk_sys::GTK_RESPONSE_YES
    }
}

impl Drop for GtkMessageDialog {
    fn drop(&mut self) {
        unsafe {
            wait_for_cleanup();
            gtk_sys::gtk_widget_destroy(self.ptr as *mut _);
            wait_for_cleanup();
        }
    }
}

impl AsGtkDialog for GtkMessageDialog {
    fn gtk_dialog_ptr(&self) -> *mut gtk_sys::GtkDialog {
        self.ptr as *mut _
    }
    unsafe fn show(&self) {
        gtk_sys::gtk_widget_show_all(self.ptr as *mut _);
    }
}

use crate::backend::MessageDialogImpl;

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> bool {
        let dialog = GtkMessageDialog::new(self);
        dialog.run()
    }
}

use crate::backend::AsyncMessageDialogImpl;
use crate::backend::DialogFutureType;

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<bool> {
        let builder = move || GtkMessageDialog::new(self);

        let future = GtkDialogFuture::new(builder, |_, res| {
            res == gtk_sys::GTK_RESPONSE_OK || res == gtk_sys::GTK_RESPONSE_YES
        });
        Box::pin(future)
    }
}
