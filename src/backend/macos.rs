use crate::{FileDialog, FileHandle};

use std::path::PathBuf;

pub use objc::runtime::{BOOL, NO};

mod policy_manager;

mod panel;
use panel::{OutputFrom, Panel};

mod async_dialog;
use async_dialog::AsyncDialog;
pub use async_dialog::DialogFuture;

use super::{
    AsyncFilePickerDialogImpl, AsyncFileSaveDialogImpl, AsyncFolderPickerDialogImpl,
    DialogFutureType, FilePickerDialogImpl, FileSaveDialogImpl, FolderPickerDialogImpl,
};

//
// File Picker
//

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

impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_pick_folder(&self);
            let res = panel.run_modal();
            OutputFrom::from(&panel, res)
        })
    }
}

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

impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_save_file(&self);
            let res = panel.run_modal();
            OutputFrom::from(&panel, res)
        })
    }
}

impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_save_file(&self);

        let ret: DialogFuture<_> = AsyncDialog::new(panel).into();
        Box::pin(ret)
    }
}

use crate::backend::MessageDialogImpl;
use crate::MessageDialog;

impl MessageDialogImpl for MessageDialog {
    fn show(self) {
        unimplemented!("");
    }
}
