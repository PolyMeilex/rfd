use crate::{FileDialog, FileHandle};

use std::path::PathBuf;

pub use objc::runtime::{BOOL, NO};

mod policy_manager;

mod panel;
use panel::{OutputFrom, Panel};

mod async_dialog;
use async_dialog::AsyncDialog;
pub use async_dialog::DialogFuture;

pub fn pick_file(opt: FileDialog) -> Option<PathBuf> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_pick_file(&opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

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

pub fn pick_files(opt: FileDialog) -> Option<Vec<PathBuf>> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_pick_files(&opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

pub fn pick_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    let panel = Panel::build_pick_file(&opt);
    AsyncDialog::new(panel).into()
}

pub fn save_file_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    let panel = Panel::build_save_file(&opt);
    AsyncDialog::new(panel).into()
}

pub fn pick_folder_async(opt: FileDialog) -> DialogFuture<Option<FileHandle>> {
    let panel = Panel::build_pick_folder(&opt);
    AsyncDialog::new(panel).into()
}

pub fn pick_files_async(opt: FileDialog) -> DialogFuture<Option<Vec<FileHandle>>> {
    let panel = Panel::build_pick_files(&opt);
    AsyncDialog::new(panel).into()
}
