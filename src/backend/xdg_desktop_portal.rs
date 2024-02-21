use std::path::PathBuf;

use super::linux::zenity;
use crate::backend::DialogFutureType;
use crate::file_dialog::Filter;
use crate::message_dialog::MessageDialog;
use crate::{FileDialog, FileHandle, MessageButtons, MessageDialogResult};

use ashpd::desktop::file_chooser::{FileFilter, OpenFileRequest, SaveFileRequest};
// TODO: convert raw_window_handle::RawWindowHandle to ashpd::WindowIdentifier
// https://github.com/bilelmoussaoui/ashpd/issues/40

use log::error;
use pollster::block_on;

impl From<&Filter> for FileFilter {
    fn from(filter: &Filter) -> Self {
        let mut ashpd_filter = FileFilter::new(&filter.name);
        for file_extension in &filter.extensions {
            ashpd_filter = ashpd_filter.glob(&format!("*.{file_extension}"));
        }
        ashpd_filter
    }
}

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        block_on(self.pick_file_async()).map(PathBuf::from)
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        block_on(self.pick_files_async())
            .map(|vec_file_handle| vec_file_handle.iter().map(PathBuf::from).collect())
    }
}

use crate::backend::AsyncFilePickerDialogImpl;
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move {
            let res = OpenFileRequest::default()
                .multiple(false)
                .title(self.title.as_deref().or(None))
                .filters(self.filters.iter().map(From::from))
                .current_folder::<&PathBuf>(&self.starting_directory)
                .expect("File path should not be nul-terminated")
                .send()
                .await;

            if res.is_err() {
                match zenity::pick_file(&self).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("pick_file error {err}");
                        return None;
                    }
                }
            } else {
                res.ok()
                    .and_then(|request| request.response().ok())
                    .and_then(|response| {
                        response
                            .uris()
                            .first()
                            .and_then(|uri| uri.to_file_path().ok())
                    })
            }
            .map(FileHandle::from)
        })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async move {
            let res = OpenFileRequest::default()
                .multiple(true)
                .title(self.title.as_deref().or(None))
                .filters(self.filters.iter().map(From::from))
                .current_folder::<&PathBuf>(&self.starting_directory)
                .expect("File path should not be nul-terminated")
                .send()
                .await;

            if res.is_err() {
                match zenity::pick_files(&self).await {
                    Ok(res) => Some(res.into_iter().map(FileHandle::from).collect::<Vec<_>>()),
                    Err(err) => {
                        error!("pick_files error {err}");
                        None
                    }
                }
            } else {
                res.ok()
                    .and_then(|request| request.response().ok())
                    .map(|response| {
                        response
                            .uris()
                            .iter()
                            .filter_map(|uri| uri.to_file_path().ok())
                            .map(FileHandle::from)
                            .collect::<Vec<FileHandle>>()
                    })
            }
        })
    }
}

//
// Folder Picker
//

use crate::backend::FolderPickerDialogImpl;
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        block_on(self.pick_folder_async()).map(PathBuf::from)
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        block_on(self.pick_folders_async())
            .map(|vec_file_handle| vec_file_handle.iter().map(PathBuf::from).collect())
    }
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move {
            let res = OpenFileRequest::default()
                .multiple(false)
                .directory(true)
                .title(self.title.as_deref().or(None))
                .filters(self.filters.iter().map(From::from))
                .current_folder::<&PathBuf>(&self.starting_directory)
                .expect("File path should not be nul-terminated")
                .send()
                .await;

            if res.is_err() {
                match zenity::pick_folder(&self).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("pick_folder error {err}");
                        return None;
                    }
                }
            } else {
                res.ok()
                    .and_then(|request| request.response().ok())
                    .and_then(|response| {
                        response
                            .uris()
                            .first()
                            .and_then(|uri| uri.to_file_path().ok())
                    })
            }
            .map(FileHandle::from)
        })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async move {
            let res = OpenFileRequest::default()
                .multiple(true)
                .directory(true)
                .title(self.title.as_deref().or(None))
                .filters(self.filters.iter().map(From::from))
                .current_folder::<&PathBuf>(&self.starting_directory)
                .expect("File path should not be nul-terminated")
                .send()
                .await;

            if res.is_err() {
                match zenity::pick_folders(&self).await {
                    Ok(res) => Some(res.into_iter().map(FileHandle::from).collect::<Vec<_>>()),
                    Err(err) => {
                        error!("pick_files error {err}");
                        None
                    }
                }
            } else {
                res.ok()
                    .and_then(|request| request.response().ok())
                    .map(|response| {
                        response
                            .uris()
                            .iter()
                            .filter_map(|uri| uri.to_file_path().ok())
                            .map(FileHandle::from)
                            .collect::<Vec<FileHandle>>()
                    })
            }
        })
    }
}

//
// File Save
//

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        block_on(self.save_file_async()).map(PathBuf::from)
    }
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async move {
            let res = SaveFileRequest::default()
                .title(self.title.as_deref().or(None))
                .current_name(self.file_name.as_deref())
                .filters(self.filters.iter().map(From::from))
                .current_folder::<&PathBuf>(&self.starting_directory)
                .expect("File path should not be nul-terminated")
                .send()
                .await;

            if res.is_err() {
                match zenity::save_file(&self).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("pick_folder error {err}");
                        return None;
                    }
                }
            } else {
                res.ok()
                    .and_then(|request| request.response().ok())
                    .and_then(|response| {
                        response
                            .uris()
                            .first()
                            .and_then(|uri| uri.to_file_path().ok())
                    })
            }
            .map(FileHandle::from)
        })
    }
}

use crate::backend::MessageDialogImpl;
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        block_on(self.show_async())
    }
}

use crate::backend::AsyncMessageDialogImpl;
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        Box::pin(async move {
            match &self.buttons {
                MessageButtons::Ok | MessageButtons::OkCustom(_) => {
                    let res = crate::backend::linux::zenity::message(
                        &self.level,
                        &self.buttons,
                        &self.title,
                        &self.description,
                    )
                    .await;

                    match res {
                        Ok(res) => res,
                        Err(err) => {
                            log::error!("Failed to open zenity dialog: {err}");
                            MessageDialogResult::Cancel
                        }
                    }
                }
                MessageButtons::OkCancel
                | MessageButtons::YesNo
                | MessageButtons::OkCancelCustom(..)
                | MessageButtons::YesNoCancel
                | MessageButtons::YesNoCancelCustom(..) => {
                    let res = crate::backend::linux::zenity::question(
                        &self.buttons,
                        &self.title,
                        &self.description,
                    )
                    .await;

                    match res {
                        Ok(res) => res,
                        Err(err) => {
                            log::error!("Failed to open zenity dialog: {err}");
                            MessageDialogResult::Cancel
                        }
                    }
                }
            }
        })
    }
}
