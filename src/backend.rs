use crate::FileHandle;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

//
// Windows
//

#[cfg(target_os = "windows")]
mod win_cid;

//
// Linux
//

#[cfg(target_os = "linux")]
mod gtk3;

//
// MacOs
//

#[cfg(target_os = "macos")]
mod macos;

//
// Wasm
//

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub type DialogFutureType<T> = Pin<Box<dyn Future<Output = T> + Send>>;
#[cfg(target_arch = "wasm32")]
pub type DialogFutureType<T> = Pin<Box<dyn Future<Output = T>>>;

//
// Sync
//

/// Dialog used to pick file/files
pub trait FilePickerDialogImpl {
    fn pick_file(self) -> Option<PathBuf>;
    fn pick_files(self) -> Option<Vec<PathBuf>>;
}

/// Dialog used to save file
pub trait FileSaveDialogImpl {
    fn save_file(self) -> Option<PathBuf>;
}

/// Dialog used to pick folder
pub trait FolderPickerDialogImpl {
    fn pick_folder(self) -> Option<PathBuf>;
}

pub trait MessageDialogImpl {
    fn show(self);
}

//
// Async
//

/// Dialog used to pick file/files
pub trait AsyncFilePickerDialogImpl {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>>;
    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>>;
}

/// Dialog used to pick folder
pub trait AsyncFolderPickerDialogImpl {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>>;
}

/// Dialog used to pick folder
pub trait AsyncFileSaveDialogImpl {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>>;
}
