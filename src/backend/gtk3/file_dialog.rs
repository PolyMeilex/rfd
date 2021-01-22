mod dialog_async;
mod dialog_ffi;

use dialog_async::{AsyncDialog, DialogFuture};
use dialog_ffi::{GtkFileDialog, OutputFrom};

use std::path::PathBuf;

use super::utils::{gtk_init_check, GTK_MUTEX};
use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        GTK_MUTEX.run_locked(|| {
            if !gtk_init_check() {
                return None;
            };

            let dialog = GtkFileDialog::build_pick_file(&self);

            let res_id = dialog.run();
            OutputFrom::from(&dialog, res_id)
        })
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        GTK_MUTEX.run_locked(|| {
            if !gtk_init_check() {
                return None;
            };

            let dialog = GtkFileDialog::build_pick_files(&self);

            let res_id = dialog.run();
            OutputFrom::from(&dialog, res_id)
        })
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let ret: DialogFuture<_> =
            AsyncDialog::new(move || GtkFileDialog::build_pick_file(&self)).into();
        Box::pin(ret)
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let ret: DialogFuture<_> =
            AsyncDialog::new(move || GtkFileDialog::build_pick_files(&self)).into();
        Box::pin(ret)
    }
}

//
// Folder Picker
//

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        GTK_MUTEX.run_locked(|| {
            if !gtk_init_check() {
                return None;
            };

            let dialog = GtkFileDialog::build_pick_folder(&self);

            let res_id = dialog.run();
            OutputFrom::from(&dialog, res_id)
        })
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let ret: DialogFuture<_> =
            AsyncDialog::new(move || GtkFileDialog::build_pick_folder(&self)).into();
        Box::pin(ret)
    }
}

//
// File Save
//

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        GTK_MUTEX.run_locked(|| {
            if !gtk_init_check() {
                return None;
            };

            let dialog = GtkFileDialog::build_save_file(&self);

            let res_id = dialog.run();
            OutputFrom::from(&dialog, res_id)
        })
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let ret: DialogFuture<_> =
            AsyncDialog::new(move || GtkFileDialog::build_save_file(&self)).into();
        Box::pin(ret)
    }
}
