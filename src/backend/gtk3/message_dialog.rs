use glib::prelude::*;
use glib::translate::{from_glib, from_glib_none, FromGlibPtrContainer};
use glib::GString;

use super::gtk_future::GtkDialogFuture;
use super::utils::GtkGlobalThread;
use super::AsGtkDialog;

use crate::message_dialog::{MessageButtons, MessageDialog, MessageLevel};
use crate::MessageDialogResult;

pub struct GtkMessageDialog {
    buttons: MessageButtons,
    object: glib::Object,
}

impl GtkMessageDialog {
    pub fn new(opt: MessageDialog) -> Self {
        let level = match opt.level {
            MessageLevel::Info => gtk_sys::GTK_MESSAGE_INFO,
            MessageLevel::Warning => gtk_sys::GTK_MESSAGE_WARNING,
            MessageLevel::Error => gtk_sys::GTK_MESSAGE_ERROR,
        };

        let buttons = match opt.buttons {
            MessageButtons::Ok => gtk_sys::GTK_BUTTONS_OK,
            MessageButtons::OkCancel => gtk_sys::GTK_BUTTONS_OK_CANCEL,
            MessageButtons::YesNo => gtk_sys::GTK_BUTTONS_YES_NO,
            MessageButtons::YesNoCancel => gtk_sys::GTK_BUTTONS_NONE,
            MessageButtons::OkCustom(_) => gtk_sys::GTK_BUTTONS_NONE,
            MessageButtons::OkCancelCustom(_, _) => gtk_sys::GTK_BUTTONS_NONE,
            MessageButtons::YesNoCancelCustom(_, _, _) => gtk_sys::GTK_BUTTONS_NONE,
        };

        let custom_buttons = match &opt.buttons {
            MessageButtons::YesNoCancel => vec![
                Some((GString::from("Yes"), gtk_sys::GTK_RESPONSE_YES)),
                Some((GString::from("No"), gtk_sys::GTK_RESPONSE_NO)),
                Some((GString::from("Cancel"), gtk_sys::GTK_RESPONSE_CANCEL)),
                None,
            ],
            MessageButtons::OkCustom(ok_text) => vec![
                Some((
                    GString::from_string_checked(ok_text.clone()).unwrap(),
                    gtk_sys::GTK_RESPONSE_OK,
                )),
                None,
            ],
            MessageButtons::OkCancelCustom(ok_text, cancel_text) => vec![
                Some((
                    GString::from_string_checked(ok_text.clone()).unwrap(),
                    gtk_sys::GTK_RESPONSE_OK,
                )),
                Some((
                    GString::from_string_checked(cancel_text.clone()).unwrap(),
                    gtk_sys::GTK_RESPONSE_CANCEL,
                )),
            ],
            MessageButtons::YesNoCancelCustom(yes_text, no_text, cancel_text) => vec![
                Some((
                    GString::from_string_checked(yes_text.clone()).unwrap(),
                    gtk_sys::GTK_RESPONSE_YES,
                )),
                Some((
                    GString::from_string_checked(no_text.clone()).unwrap(),
                    gtk_sys::GTK_RESPONSE_NO,
                )),
                Some((
                    GString::from_string_checked(cancel_text.clone()).unwrap(),
                    gtk_sys::GTK_RESPONSE_CANCEL,
                )),
                None,
            ],
            _ => vec![],
        };

        let title = GString::from_string_checked(opt.title.clone()).unwrap();
        let description = GString::from_string_checked(opt.description.clone()).unwrap();

        let object: glib::Object = unsafe {
            let parent_gtk_window = opt
                .parent
                .as_ref()
                .and_then(|parent_handle| super::utils::find_gtk_window(parent_handle));
            let parent_gtk_window = parent_gtk_window
                .as_ref()
                .map_or(std::ptr::null_mut(), |p| {
                    p.as_ptr() as *mut gtk_sys::GtkWindow
                });

            let dialog = gtk_sys::gtk_message_dialog_new(
                parent_gtk_window,
                gtk_sys::GTK_DIALOG_MODAL,
                level,
                buttons,
                b"%s\0".as_ptr() as *mut _,
                title.as_ptr(),
            ) as *mut gtk_sys::GtkDialog;

            set_child_labels_selectable(dialog);
            // Also set the window title, otherwise it would be empty
            gtk_sys::gtk_window_set_title(dialog as _, title.as_ptr());

            for custom_button in custom_buttons {
                if let Some((custom_button_cstr, response_id)) = custom_button {
                    gtk_sys::gtk_dialog_add_button(
                        dialog,
                        custom_button_cstr.as_ptr(),
                        response_id,
                    );
                }
            }

            gtk_sys::gtk_message_dialog_format_secondary_text(
                dialog as *mut _,
                c"%s".as_ptr(),
                description.as_ptr(),
            );

            // GTK keeps the toplevel window reference to itself, so take our own
            from_glib_none(dialog as *mut glib::gobject_ffi::GObject)
        };

        Self {
            object,
            buttons: opt.buttons,
        }
    }

    fn ptr(&self) -> *mut gtk_sys::GtkDialog {
        self.object.as_ptr() as *mut _
    }

    pub fn run(self) -> MessageDialogResult {
        let res = unsafe { gtk_sys::gtk_dialog_run(self.ptr()) };

        use MessageButtons::*;
        match (&self.buttons, res) {
            (Ok | OkCancel, gtk_sys::GTK_RESPONSE_OK) => MessageDialogResult::Ok,
            (Ok | OkCancel | YesNoCancel, gtk_sys::GTK_RESPONSE_CANCEL) => {
                MessageDialogResult::Cancel
            }
            (YesNo | YesNoCancel, gtk_sys::GTK_RESPONSE_YES) => MessageDialogResult::Yes,
            (YesNo | YesNoCancel, gtk_sys::GTK_RESPONSE_NO) => MessageDialogResult::No,
            (OkCustom(custom), gtk_sys::GTK_RESPONSE_OK) => {
                MessageDialogResult::Custom(custom.to_owned())
            }
            (OkCancelCustom(custom, _), gtk_sys::GTK_RESPONSE_OK) => {
                MessageDialogResult::Custom(custom.to_owned())
            }
            (OkCancelCustom(_, custom), gtk_sys::GTK_RESPONSE_CANCEL) => {
                MessageDialogResult::Custom(custom.to_owned())
            }
            (YesNoCancelCustom(custom, _, _), gtk_sys::GTK_RESPONSE_YES) => {
                MessageDialogResult::Custom(custom.to_owned())
            }
            (YesNoCancelCustom(_, custom, _), gtk_sys::GTK_RESPONSE_NO) => {
                MessageDialogResult::Custom(custom.to_owned())
            }
            (YesNoCancelCustom(_, _, custom), gtk_sys::GTK_RESPONSE_CANCEL) => {
                MessageDialogResult::Custom(custom.to_owned())
            }
            _ => MessageDialogResult::Cancel,
        }
    }
}

/// Sets the child labels of a widget selectable
unsafe fn set_child_labels_selectable(dialog: *mut gtk_sys::GtkDialog) {
    let label_type: glib::Type = from_glib(gtk_sys::gtk_label_get_type());

    let area = gtk_sys::gtk_message_dialog_get_message_area(dialog as _);
    let children: Vec<glib::Object> =
        FromGlibPtrContainer::from_glib_container(gtk_sys::gtk_container_get_children(area as _));

    for child in children {
        if child.type_().is_a(label_type) {
            gtk_sys::gtk_label_set_selectable(child.as_ptr() as _, glib::ffi::GTRUE);
        }
    }
}

impl Drop for GtkMessageDialog {
    fn drop(&mut self) {
        unsafe {
            gtk_sys::gtk_widget_destroy(self.ptr() as *mut _);
        }
    }
}

impl AsGtkDialog for GtkMessageDialog {
    fn object(&self) -> &glib::Object {
        &self.object
    }
    unsafe fn show(&self) {
        gtk_sys::gtk_widget_show_all(self.ptr() as *mut _);
    }
}

use crate::backend::MessageDialogImpl;

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        GtkGlobalThread::instance().run_blocking(move || {
            let dialog = GtkMessageDialog::new(self);
            dialog.run()
        })
    }
}

use crate::backend::AsyncMessageDialogImpl;
use crate::backend::DialogFutureType;

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        let builder = move || GtkMessageDialog::new(self);

        let future = GtkDialogFuture::new(builder, |_, res| match res {
            gtk_sys::GTK_RESPONSE_OK => MessageDialogResult::Ok,
            gtk_sys::GTK_RESPONSE_CANCEL => MessageDialogResult::Cancel,
            gtk_sys::GTK_RESPONSE_YES => MessageDialogResult::Yes,
            gtk_sys::GTK_RESPONSE_NO => MessageDialogResult::No,
            gtk_sys::GTK_RESPONSE_DELETE_EVENT => MessageDialogResult::Cancel,
            _ => unreachable!(),
        });
        Box::pin(future)
    }
}
