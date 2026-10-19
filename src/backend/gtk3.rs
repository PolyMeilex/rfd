mod file_dialog;
mod message_dialog;

mod utils;

trait AsGtkDialog {
    fn gtk_dialog_ptr(&self) -> *mut gtk_sys::GtkDialog;
    unsafe fn show(&self);
}
