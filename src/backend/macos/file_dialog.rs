mod dialog_async;
mod dialog_ffi;

use dialog_async::{AsyncDialog, DialogFuture};
use dialog_ffi::{OutputFrom, Panel};

use crate::backend::DialogFutureType;

use crate::{FileDialog, FileHandle};

use std::path::PathBuf;

pub use objc::runtime::{BOOL, NO};

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_pick_file(&self);

            let res = panel.run_modal();
            OutputFrom::from(&panel, res)
        })
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_pick_files(&self);

            let res = panel.run_modal();
            OutputFrom::from(&panel, res)
        })
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_pick_file(&self);

        let ret: DialogFuture<_> = AsyncDialog::new(panel).into();
        Box::pin(ret)
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let panel = Panel::build_pick_files(&self);

        let ret: DialogFuture<_> = AsyncDialog::new(panel).into();
        Box::pin(ret)
    }
}

//
// Folder Picker
//

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_pick_folder(&self);
            let res = panel.run_modal();
            OutputFrom::from(&panel, res)
        })
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_pick_folder(&self);
        let ret: DialogFuture<_> = AsyncDialog::new(panel).into();
        Box::pin(ret)
    }
}

//
// File Save
//

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_save_file(&self);
            let res = panel.run_modal();
            OutputFrom::from(&panel, res)
        })
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_save_file(&self);

        let ret: DialogFuture<_> = AsyncDialog::new(panel).into();
        Box::pin(ret)
    }
}

use crate::MessageDialog;

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) {
        unimplemented!("");
    }
}
