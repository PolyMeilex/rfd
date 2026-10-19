mod file_dialog;
mod message_dialog;

mod utils;

trait AsGtkDialog {
    fn object(&self) -> &glib::Object;
    unsafe fn show(&self);
}
