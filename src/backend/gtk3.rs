use crate::FileDialog;
use crate::FileHandle;

use std::pin::Pin;
use std::{future::Future, path::PathBuf, ptr};

mod dialog_future;
use dialog_future::{AsyncDialog, DialogFuture};

mod file_dialog;
use file_dialog::{GtkFileDialog, OutputFrom};

mod message_dialog;

mod gtk_guard;
use gtk_guard::GTK_MUTEX;

// pub fn pick_file(opt: FileDialog) -> Option<PathBuf> {
//     GTK_MUTEX.run_locked(|| {
//         if !gtk_init_check() {
//             return None;
//         };

//         let dialog = GtkFileDialog::build_pick_file(&opt);

//         let res_id = dialog.run();
//         OutputFrom::from(&dialog, res_id)
//     })
// }

// pub fn save_file(opt: FileDialog) -> Option<PathBuf> {
//     GTK_MUTEX.run_locked(|| {
//         if !gtk_init_check() {
//             return None;
//         };

//         let dialog = GtkFileDialog::build_save_file(&opt);

//         let res_id = dialog.run();
//         OutputFrom::from(&dialog, res_id)
//     })
// }

// pub fn pick_folder(opt: FileDialog) -> Option<PathBuf> {
//     GTK_MUTEX.run_locked(|| {
//         if !gtk_init_check() {
//             return None;
//         };

//         let dialog = GtkFileDialog::build_pick_folder(&opt);

//         let res_id = dialog.run();
//         OutputFrom::from(&dialog, res_id)
//     })
// }

// pub fn pick_files(opt: FileDialog) -> Option<Vec<PathBuf>> {
//     GTK_MUTEX.run_locked(|| {
//         if !gtk_init_check() {
//             return None;
//         };

//         let dialog = GtkFileDialog::build_pick_files(&opt);

//         let res_id = dialog.run();
//         OutputFrom::from(&dialog, res_id)
//     })
// }

use super::{
    AsyncFilePickerDialogImpl, AsyncFileSaveDialogImpl, AsyncFolderPickerDialogImpl,
    DialogFutureType, FilePickerDialogImpl, FileSaveDialogImpl, FolderPickerDialogImpl,
};

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

impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let ret: DialogFuture<_> =
            AsyncDialog::new(move || GtkFileDialog::build_pick_folder(&self)).into();
        Box::pin(ret)
    }
}

//
//
//

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

impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let ret: DialogFuture<_> =
            AsyncDialog::new(move || GtkFileDialog::build_save_file(&self)).into();
        Box::pin(ret)
    }
}

//
//
//

// pub fn pick_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
//     AsyncDialog::new(move || GtkFileDialog::build_pick_file(&opt)).into()
// }

// pub fn save_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
//     AsyncDialog::new(move || GtkFileDialog::build_save_file(&opt)).into()
// }

// pub fn pick_folder_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
//     AsyncDialog::new(move || GtkFileDialog::build_pick_folder(&opt)).into()
// }

// pub fn pick_files_async(opt: FileDialog) -> DialogFuture<Option<Vec<FileHandle>>> {
//     AsyncDialog::new(move || GtkFileDialog::build_pick_files(&opt)).into()
// }

//
//
//

fn gtk_init_check() -> bool {
    unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1 }
}

/// gtk_main_iteration()
unsafe fn wait_for_cleanup() {
    while gtk_sys::gtk_events_pending() == 1 {
        gtk_sys::gtk_main_iteration();
    }
}
