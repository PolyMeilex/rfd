use crate::FileDialog;
use crate::FileHandle;

use std::{path::PathBuf, ptr};

mod dialog_future;
use dialog_future::{AsyncDialog, DialogFuture};

mod file_dialog;
use file_dialog::{GtkFileDialog, OutputFrom};

mod message_dialog;

mod gtk_guard;
use gtk_guard::GTK_MUTEX;

pub fn pick_file(opt: FileDialog) -> Option<PathBuf> {
    GTK_MUTEX.run_locked(|| {
        if !gtk_init_check() {
            return None;
        };

        let dialog = GtkFileDialog::build_pick_file(&opt);

        let res_id = dialog.run();
        OutputFrom::from(&dialog, res_id)
    })
}

pub fn save_file(opt: FileDialog) -> Option<PathBuf> {
    GTK_MUTEX.run_locked(|| {
        if !gtk_init_check() {
            return None;
        };

        let dialog = GtkFileDialog::build_save_file(&opt);

        let res_id = dialog.run();
        OutputFrom::from(&dialog, res_id)
    })
}

pub fn pick_folder(opt: FileDialog) -> Option<PathBuf> {
    GTK_MUTEX.run_locked(|| {
        if !gtk_init_check() {
            return None;
        };

        let dialog = GtkFileDialog::build_pick_folder(&opt);

        let res_id = dialog.run();
        OutputFrom::from(&dialog, res_id)
    })
}

pub fn pick_files(opt: FileDialog) -> Option<Vec<PathBuf>> {
    GTK_MUTEX.run_locked(|| {
        if !gtk_init_check() {
            return None;
        };

        let dialog = GtkFileDialog::build_pick_files(&opt);

        let res_id = dialog.run();
        OutputFrom::from(&dialog, res_id)
    })
}

//
//
//

pub fn pick_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    AsyncDialog::new(move || GtkFileDialog::build_pick_file(&opt)).into()
}

pub fn save_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    AsyncDialog::new(move || GtkFileDialog::build_save_file(&opt)).into()
}

pub fn pick_folder_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    AsyncDialog::new(move || GtkFileDialog::build_pick_folder(&opt)).into()
}

pub fn pick_files_async(opt: FileDialog) -> DialogFuture<Option<Vec<FileHandle>>> {
    AsyncDialog::new(move || GtkFileDialog::build_pick_files(&opt)).into()
}

//
//
//

fn gtk_init_check() -> bool {
    unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1 }
}
