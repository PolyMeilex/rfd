use objc2::rc::autoreleasepool;
use objc2_app_kit::NSModalResponseOK;
use std::path::PathBuf;

mod panel_ffi;

use self::panel_ffi::Panel;
use super::modal_future::ModalFuture;
use super::utils::run_on_main;
use crate::backend::{
    AsyncFileOrFolderPickerDialogImpl, AsyncFilePickerDialogImpl, AsyncFileSaveDialogImpl,
    AsyncFolderPickerDialogImpl, DialogFutureType, FileOrFolderPickerDialogImpl,
    FilePickerDialogImpl, FileSaveDialogImpl, FolderPickerDialogImpl,
};
use crate::{FileDialog, FileHandle};

//
// File Picker
//

impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_pick_file(&self, mtm);

                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_result())
                } else {
                    None
                }
            })
        })
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_pick_files(&self, mtm);

                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_results())
                } else {
                    None
                }
            })
        })
    }
}

impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_pick_file(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
                    Some(panel.get_result().into())
                } else {
                    None
                }
            },
        );

        Box::pin(future)
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_pick_files(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
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
            },
        );

        Box::pin(future)
    }
}

//
// Folder Picker
//

impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_pick_folder(&self, mtm);
                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_result())
                } else {
                    None
                }
            })
        })
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_pick_folders(&self, mtm);
                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_results())
                } else {
                    None
                }
            })
        })
    }
}

//
// File or Folder Picker
//

impl FileOrFolderPickerDialogImpl for FileDialog {
    fn pick_file_or_folder(self) -> Option<PathBuf> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_pick_file_or_folder(&self, mtm);
                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_result())
                } else {
                    None
                }
            })
        })
    }

    fn pick_files_or_folders(self) -> Option<Vec<PathBuf>> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_pick_files_or_folders(&self, mtm);
                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_results())
                } else {
                    None
                }
            })
        })
    }
}

impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_pick_folder(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
                    Some(panel.get_result().into())
                } else {
                    None
                }
            },
        );

        Box::pin(future)
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_pick_folders(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
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
            },
        );

        Box::pin(future)
    }
}

impl AsyncFileOrFolderPickerDialogImpl for FileDialog {
    fn pick_file_or_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_pick_file_or_folder(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
                    Some(panel.get_result().into())
                } else {
                    None
                }
            },
        );

        Box::pin(future)
    }

    fn pick_files_or_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_pick_files_or_folders(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
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
            },
        );

        Box::pin(future)
    }
}

//
// File Save
//

impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        autoreleasepool(move |_| {
            run_on_main(move |mtm| {
                let panel = Panel::build_save_file(&self, mtm);
                if panel.run_modal() == NSModalResponseOK {
                    Some(panel.get_result())
                } else {
                    None
                }
            })
        })
    }
}

impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let future = ModalFuture::new(
            move |mtm| Panel::build_save_file(&self, mtm),
            |panel, res_id| {
                if res_id == NSModalResponseOK {
                    Some(panel.get_result().into())
                } else {
                    None
                }
            },
        );

        Box::pin(future)
    }
}
