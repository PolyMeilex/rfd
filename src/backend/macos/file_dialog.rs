mod panel_ffi;
use panel_ffi::Panel;

use crate::backend::DialogFutureType;
use crate::{FileDialog, FileHandle};

use std::path::PathBuf;

pub use objc::runtime::{BOOL, NO};

use super::modal_future::ModalFuture;

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_pick_file(&self);

            if panel.run_modal() == 1 {
                Some(panel.get_result())
            } else {
                None
            }
        })
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        objc::rc::autoreleasepool(move || {
            let panel = Panel::build_pick_files(&self);

            if panel.run_modal() == 1 {
                Some(panel.get_results())
            } else {
                None
            }
        })
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_pick_file(&self);

        let future = ModalFuture::new(panel, |panel, res_id| {
            if res_id == 1 {
                Some(panel.get_result().into())
            } else {
                None
            }
        });

        Box::pin(future)
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let panel = Panel::build_pick_files(&self);

        let future = ModalFuture::new(panel, |panel, res_id| {
            if res_id == 1 {
                Some(
                    panel
                        .get_results()
                        .into_iter()
                        .map(FileHandle::wrap)
                        .collect(),
                )
            } else {
                None
            }
        });

        Box::pin(future)
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
            if panel.run_modal() == 1 {
                Some(panel.get_result())
            } else {
                None
            }
        })
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_pick_folder(&self);

        let future = ModalFuture::new(panel, |panel, res_id| {
            if res_id == 1 {
                Some(panel.get_result().into())
            } else {
                None
            }
        });

        Box::pin(future)
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
            if panel.run_modal() == 1 {
                Some(panel.get_result())
            } else {
                None
            }
        })
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let panel = Panel::build_save_file(&self);

        let future = ModalFuture::new(panel, |panel, res_id| {
            if res_id == 1 {
                Some(panel.get_result().into())
            } else {
                None
            }
        });

        Box::pin(future)
    }
}
