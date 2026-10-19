pub mod dialog_ffi;

use dialog_ffi::GtkFileDialog;

use std::path::PathBuf;

use super::utils::{run_dialog, spawn_dialog};
use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

fn result(dialog: &GtkFileDialog, res_id: i32) -> Option<PathBuf> {
    if res_id == gtk_sys::GTK_RESPONSE_ACCEPT {
        dialog.get_result()
    } else {
        None
    }
}

fn results(dialog: &GtkFileDialog, res_id: i32) -> Option<Vec<PathBuf>> {
    if res_id == gtk_sys::GTK_RESPONSE_ACCEPT {
        Some(dialog.get_results())
    } else {
        None
    }
}

fn wrap_results(paths: Option<Vec<PathBuf>>) -> Option<Vec<FileHandle>> {
    paths.map(|paths| paths.into_iter().map(FileHandle::wrap).collect())
}

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        run_dialog(move || GtkFileDialog::build_pick_file(&self), result)
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        run_dialog(move || GtkFileDialog::build_pick_files(&self), results)
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        spawn_dialog(
            move || GtkFileDialog::build_pick_file(&self),
            |dialog, res_id| result(dialog, res_id).map(FileHandle::wrap),
        )
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        spawn_dialog(
            move || GtkFileDialog::build_pick_files(&self),
            |dialog, res_id| wrap_results(results(dialog, res_id)),
        )
    }
}

//
// Folder Picker
//

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        run_dialog(move || GtkFileDialog::build_pick_folder(&self), result)
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        run_dialog(move || GtkFileDialog::build_pick_folders(&self), results)
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        spawn_dialog(
            move || GtkFileDialog::build_pick_folder(&self),
            |dialog, res_id| result(dialog, res_id).map(FileHandle::wrap),
        )
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        spawn_dialog(
            move || GtkFileDialog::build_pick_folders(&self),
            |dialog, res_id| wrap_results(results(dialog, res_id)),
        )
    }
}

//
// File Save
//

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        run_dialog(move || GtkFileDialog::build_save_file(&self), result)
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        spawn_dialog(
            move || GtkFileDialog::build_save_file(&self),
            |dialog, res_id| result(dialog, res_id).map(FileHandle::wrap),
        )
    }
}
