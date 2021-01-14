mod backend;

#[cfg(target_os = "macos")]
pub use backend::macos;
#[cfg(target_arch = "wasm32")]
pub use backend::wasm;

#[cfg(any(target_arch = "wasm32", feature = "native-file-handle"))]
mod file_handle;
#[cfg(any(target_arch = "wasm32", feature = "native-file-handle"))]
pub use file_handle::FileHandle;

mod dialog;
pub use dialog::{FileDialog, Filter};
