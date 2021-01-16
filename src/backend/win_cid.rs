//! Windows Common Item Dialog
//! Win32 Vista
use crate::FileDialog;
use crate::FileHandle;

use std::path::PathBuf;

use winapi::shared::winerror::HRESULT;

mod util;
use util::init_com;

mod win_dialog;
use win_dialog::Dialog;

mod async_dialog;
pub use async_dialog::DialogFuture;

pub fn pick_file(opt: FileDialog) -> Option<PathBuf> {
    fn run(opt: FileDialog) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::build_pick_file(&opt)?;
            dialog.show()?;
            dialog.get_result()
        })?
    }

    run(opt).ok()
}

pub fn save_file(opt: FileDialog) -> Option<PathBuf> {
    fn run(opt: FileDialog) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::build_save_file(&opt)?;
            dialog.show()?;
            dialog.get_result()
        })?
    }

    run(opt).ok()
}

pub fn pick_folder(opt: FileDialog) -> Option<PathBuf> {
    fn run(opt: FileDialog) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::build_pick_folder(&opt)?;
            dialog.show()?;
            dialog.get_result()
        })?
    }

    run(opt).ok()
}

pub fn pick_files(opt: FileDialog) -> Option<Vec<PathBuf>> {
    fn run(opt: FileDialog) -> Result<Vec<PathBuf>, HRESULT> {
        init_com(|| {
            let dialog = Dialog::build_pick_files(&opt)?;
            dialog.show()?;
            dialog.get_results()
        })?
    }

    run(opt).ok()
}

//
//
//

pub fn pick_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    unimplemented!("")
}

pub fn save_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    unimplemented!("")
}

pub fn pick_folder_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    unimplemented!("")
}

pub fn pick_files_async(opt: FileDialog) -> DialogFuture<Option<Vec<FileHandle>>> {
    unimplemented!("")
}
