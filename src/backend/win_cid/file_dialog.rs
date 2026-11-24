mod com;
pub mod dialog_ffi;
mod save_extension;

use dialog_ffi::{IDialog, Result};

use crate::backend::DialogFutureType;
use crate::FileDialog;
use crate::FileHandle;

use std::path::PathBuf;

use super::utils::init_com;

fn async_thread<T, F>(f: F) -> DialogFutureType<Option<T>>
where
    F: FnOnce() -> Option<T>,
    F: Send + 'static,
    T: Send + 'static,
{
    Box::pin(async move {
        let (tx, rx) = crate::oneshot::channel();

        std::thread::spawn(move || {
            tx.send(f()).ok();
        });

        rx.await.ok()?
    })
}

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        fn run(opt: FileDialog) -> Result<PathBuf> {
            init_com(|| {
                let dialog = IDialog::build_pick_file(&opt)?;
                dialog.show()?;
                dialog.get_result()
            })?
        }
        run(self).ok()
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        fn run(opt: FileDialog) -> Result<Vec<PathBuf>> {
            init_com(|| {
                let dialog = IDialog::build_pick_files(&opt)?;
                dialog.show()?;
                dialog.get_results()
            })?
        }
        run(self).ok()
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::pick_file(self).map(FileHandle::wrap))
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        async_thread(move || {
            Self::pick_files(self).map(|res| res.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

//
// Folder Picker
//

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        fn run(opt: FileDialog) -> Result<PathBuf> {
            init_com(|| {
                let dialog = IDialog::build_pick_folder(&opt)?;
                dialog.show()?;
                dialog.get_result()
            })?
        }

        run(self).ok()
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        fn run(opt: FileDialog) -> Result<Vec<PathBuf>> {
            init_com(|| {
                let dialog = IDialog::build_pick_folders(&opt)?;
                dialog.show()?;
                dialog.get_results()
            })?
        }
        run(self).ok()
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::pick_folder(self).map(FileHandle::wrap))
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        async_thread(move || {
            Self::pick_folders(self).map(|res| res.into_iter().map(FileHandle::wrap).collect())
        })
    }
}

//
// File Save
//

/// The path the save dialog returned, with the extension of the file type that was selected.
///
/// Changing the file type in the dialog doesn't always change the extension of the name, so the
/// path can still have the extension of the name given to `set_file_name`.
fn save_result(dialog: &IDialog, opt: &FileDialog) -> Result<PathBuf> {
    let path = dialog.get_result()?;
    // Without the index the path stays as the dialog returned it
    let index = dialog.get_file_type_index().ok();
    Ok(with_selected_file_type(path, opt, index))
}

/// `index` is the 1-based index of the selected file type in `opt.filters`.
fn with_selected_file_type(path: PathBuf, opt: &FileDialog, index: Option<u32>) -> PathBuf {
    let renamed = save_extension::with_selected_file_type(
        &path,
        opt.file_name.as_deref(),
        opt.filters
            .iter()
            .map(|filter| filter.extensions.as_slice()),
        index,
    );
    match renamed {
        // The dialog only asked to overwrite the path it returned
        Some(renamed) if !renamed.exists() => renamed,
        _ => path,
    }
}

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        fn run(opt: FileDialog) -> Result<PathBuf> {
            init_com(|| {
                let dialog = IDialog::build_save_file(&opt)?;
                dialog.show()?;
                save_result(&dialog, &opt)
            })?
        }

        run(self).ok()
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::save_file(self).map(FileHandle::wrap))
    }
}
