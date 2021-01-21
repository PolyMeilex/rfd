use crate::{FileDialog, FileHandle};

use std::path::PathBuf;

pub use objc::runtime::{BOOL, NO};

mod policy_manager;

mod panel;
use panel::{OutputFrom, Panel};

mod async_dialog;
use async_dialog::AsyncDialog;
pub use async_dialog::DialogFuture;

// pub fn pick_file(opt: FileDialog) -> Option<PathBuf> {
//     objc::rc::autoreleasepool(move || {
//         let panel = Panel::build_pick_file(&opt);

//         let res = panel.run_modal();
//         OutputFrom::from(&panel, res)
//     })
// }

pub fn save_file(opt: FileDialog) -> Option<PathBuf> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_save_file(&opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

pub fn pick_folder(opt: FileDialog) -> Option<PathBuf> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_pick_folder(&opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

// pub fn pick_files(opt: FileDialog) -> Option<Vec<PathBuf>> {
//     objc::rc::autoreleasepool(move || {
//         let panel = Panel::build_pick_files(&opt);

//         let res = panel.run_modal();
//         OutputFrom::from(&panel, res)
//     })
// }

// pub fn pick_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
//     let panel = Panel::build_pick_file(&opt);
//     AsyncDialog::new(panel).into()
// }

pub fn save_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    let panel = Panel::build_save_file(&opt);
    AsyncDialog::new(panel).into()
}

pub fn pick_folder_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    let panel = Panel::build_pick_folder(&opt);
    AsyncDialog::new(panel).into()
}

// pub fn pick_files_async(opt: FileDialog) -> DialogFuture<Option<Vec<FileHandle>>> {
//     let panel = Panel::build_pick_files(&opt);
//     AsyncDialog::new(panel).into()
// }

use super::{AsyncFilePickerDialogImpl, DialogFutureType, FilePickerDialogImpl};

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

use crate::backend::MessageDialogImpl;
use crate::MessageDialog;

impl MessageDialogImpl for MessageDialog {
    fn show(self) {
        unimplemented!("");
    }
}
