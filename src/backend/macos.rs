use crate::FileDialog;
use std::path::PathBuf;

pub use objc::runtime::{BOOL, NO};

mod policy_manager;

mod panel;
use panel::{OutputFrom, Panel};

mod async_dialog;
use async_dialog::{AsyncDialog, DialogFuture};

pub fn pick_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_pick_file(opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

pub fn save_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_save_file(opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

pub fn pick_folder<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_pick_folder(opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

pub fn pick_files<'a>(opt: &FileDialog<'a>) -> Option<Vec<PathBuf>> {
    objc::rc::autoreleasepool(move || {
        let panel = Panel::build_pick_files(opt);

        let res = panel.run_modal();
        OutputFrom::from(&panel, res)
    })
}

pub fn async_test() -> DialogFuture<Option<PathBuf>> {
    let panel = Panel::open_panel();
    AsyncDialog::new(panel).into()
}
