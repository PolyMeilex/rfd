//! Windows Common Item Dialog
//! Win32 Vista
use crate::FileDialog;
use crate::FileHandle;

use std::{path::PathBuf, ptr};

use winapi::{
    shared::winerror::HRESULT,
    um::shobjidl::{FOS_ALLOWMULTISELECT, FOS_PICKFOLDERS},
};

mod util;
use util::{init_com, ToResult};

mod win_dialog;
use win_dialog::Dialog;

mod async_dialog;
pub use async_dialog::DialogFuture;

pub fn pick_file(opt: FileDialog) -> Option<PathBuf> {
    unsafe fn run(opt: FileDialog) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            dialog.add_filters(&opt.filters)?;
            dialog.set_path(&opt.starting_directory)?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_result()
        })?
    }

    unsafe { run(opt).ok() }
}

pub fn save_file(opt: FileDialog) -> Option<PathBuf> {
    unsafe fn run(opt: FileDialog) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_save_dialog()?;

            dialog.add_filters(&opt.filters)?;
            dialog.set_path(&opt.starting_directory)?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_result()
        })?
    }

    unsafe { run(opt).ok() }
}

pub fn pick_folder(opt: FileDialog) -> Option<PathBuf> {
    unsafe fn run(opt: FileDialog) -> Result<PathBuf, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            dialog.set_path(&opt.starting_directory)?;

            dialog.SetOptions(FOS_PICKFOLDERS).check()?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_result()
        })?
    }

    unsafe { run(opt).ok() }
}

pub fn pick_files(opt: FileDialog) -> Option<Vec<PathBuf>> {
    unsafe fn run(opt: FileDialog) -> Result<Vec<PathBuf>, HRESULT> {
        init_com(|| {
            let dialog = Dialog::new_open_dialog()?;

            dialog.add_filters(&opt.filters)?;
            dialog.set_path(&opt.starting_directory)?;

            dialog.SetOptions(FOS_ALLOWMULTISELECT).check()?;

            dialog.Show(ptr::null_mut()).check()?;

            dialog.get_results()
        })?
    }

    unsafe { run(opt).ok() }
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
