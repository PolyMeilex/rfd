mod file_dialog;
mod message_dialog;

mod gtk_future;

mod utils;

pub(self) trait AsGtkDialog {
    fn object(&self) -> &glib::Object;
    unsafe fn show(&self);
}
