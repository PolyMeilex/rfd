use crate::FileHandle;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

//
// Windows
//

#[cfg(target_os = "windows")]
mod win_cid;
#[cfg(target_os = "windows")]
pub use win_cid::{pick_folder, save_file};
#[cfg(target_os = "windows")]
pub use win_cid::{pick_folder_async, save_file_async};

//
// Linux
//

#[cfg(target_os = "linux")]
mod gtk3;
#[cfg(target_os = "linux")]
pub use gtk3::{pick_folder, save_file};
#[cfg(target_os = "linux")]
pub use gtk3::{pick_folder_async, save_file_async};

//
// MacOs
//

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{pick_file, pick_files, pick_folder, save_file};
#[cfg(target_os = "macos")]
pub use macos::{pick_file_async, pick_files_async, pick_folder_async, save_file_async};

//
// Wasm
//

#[cfg(target_arch = "wasm32")]
pub mod wasm;
// #[cfg(target_arch = "wasm32")]
// pub use wasm::{pick_file, pick_files, pick_folder, save_file};
#[cfg(target_arch = "wasm32")]
pub use wasm::{pick_file_async, pick_files_async /*pick_folder_async*/ /*save_file_async*/};

#[cfg(not(target_arch = "wasm32"))]
pub type DialogFutureType<T> = Pin<Box<dyn Future<Output = T> + Send>>;
#[cfg(target_arch = "wasm32")]
pub type DialogFutureType<T> = Pin<Box<dyn Future<Output = T>>>;

pub trait FilePickerDialogImpl {
    fn pick_file(self) -> Option<PathBuf>;
    fn pick_files(self) -> Option<Vec<PathBuf>>;
}

pub trait AsyncFilePickerDialogImpl {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>>;
    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>>;
}

#[cfg(test)]
mod tests {
    #[test]
    /// Check if all fns are defined
    #[allow(unused_imports)]
    fn fn_def_check() {
        // Sync

        #[cfg(not(target_arch = "wasm32"))]
        use super::{pick_folder, save_file};

        // Async

        #[cfg(not(target_arch = "wasm32"))]
        use super::{pick_folder_async, save_file_async};
    }
}
