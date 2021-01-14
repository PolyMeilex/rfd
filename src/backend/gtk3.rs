use crate::FileDialog;
use crate::FileHandle;

use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    ptr,
};

mod async_dialog;
use async_dialog::{AsyncDialog, DialogFuture};

mod gtk_dialog;
pub(crate) use gtk_dialog::{GtkDialog, GtkFileChooserAction, OutputFrom};

pub fn pick_file<'a>(opt: FileDialog) -> Option<PathBuf> {
    if !gtk_init_check() {
        return None;
    };

    let dialog = GtkDialog::build_pick_file(&opt);

    let res_id = dialog.run();
    OutputFrom::from(&dialog, res_id)
}

pub fn save_file<'a>(opt: FileDialog) -> Option<PathBuf> {
    if !gtk_init_check() {
        return None;
    };

    let dialog = GtkDialog::build_save_file(&opt);

    let res_id = dialog.run();
    OutputFrom::from(&dialog, res_id)
}

pub fn pick_folder<'a>(opt: FileDialog) -> Option<PathBuf> {
    if !gtk_init_check() {
        return None;
    };

    let dialog = GtkDialog::build_pick_folder(&opt);

    let res_id = dialog.run();
    OutputFrom::from(&dialog, res_id)
}

pub fn pick_files<'a>(opt: FileDialog) -> Option<Vec<PathBuf>> {
    if !gtk_init_check() {
        return None;
    };

    let dialog = GtkDialog::build_pick_files(&opt);

    let res_id = dialog.run();
    OutputFrom::from(&dialog, res_id)
}

//
//
//

pub fn pick_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    AsyncDialog::new(move || GtkDialog::build_pick_file(&opt)).into()
}

pub fn save_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    AsyncDialog::new(move || GtkDialog::build_save_file(&opt)).into()
}

pub fn pick_folder_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    AsyncDialog::new(move || GtkDialog::build_pick_folder(&opt)).into()
}

pub fn pick_files_async(opt: FileDialog) -> DialogFuture<Option<Vec<FileHandle>>> {
    AsyncDialog::new(move || GtkDialog::build_pick_files(&opt)).into()
}

//
//
//

fn gtk_init_check() -> bool {
    unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1 }
}
