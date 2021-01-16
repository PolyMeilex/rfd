mod backend;

#[cfg(target_arch = "wasm32")]
pub use backend::wasm;

mod file_handle;
pub use file_handle::FileHandle;

mod dialog;

#[cfg(not(target_arch = "wasm32"))]
pub use dialog::FileDialog;

pub use dialog::AsyncFileDialog;
