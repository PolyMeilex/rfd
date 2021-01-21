mod backend;

mod file_handle;
pub use file_handle::FileHandle;

mod dialog;

#[cfg(not(target_arch = "wasm32"))]
pub use dialog::FileDialog;

pub use dialog::AsyncFileDialog;

pub use dialog::MessageDialog;

trait MessageDialogImpl {
    fn show(self);
}
