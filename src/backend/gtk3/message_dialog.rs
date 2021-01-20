use std::ptr;

pub struct GtkMessageDialog {}

impl GtkMessageDialog {
    pub fn new() -> Self {
        super::gtk_init_check();

        unsafe {
            let dialog = gtk_sys::gtk_message_dialog_new(
                ptr::null_mut(),
                gtk_sys::GTK_DIALOG_MODAL,
                gtk_sys::GTK_MESSAGE_INFO,
                gtk_sys::GTK_BUTTONS_OK,
                b"%s\0".as_ptr() as *mut _,
                b"test\0",
            );
            gtk_sys::gtk_dialog_run(dialog as *mut _);
        }
        Self {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // super::GtkMessageDialog::new();
    }
}
