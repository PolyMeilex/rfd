use std::path::PathBuf;

use super::linux::zenity;
use crate::backend::xdg_impl::{
    desktop::{
        FileFilter, FilePath, OpenFileOptions, OpenFileRequest, SaveFileOptions, SaveFileRequest,
    },
    WindowIdentifier,
};
use crate::backend::DialogFutureType;
use crate::file_dialog::Filter;
use crate::message_dialog::MessageDialog;
use crate::{FileDialog, FileHandle, MessageButtons, MessageDialogResult};

use log::error;
use pollster::block_on;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

fn to_window_identifier(
    window: Option<RawWindowHandle>,
    display: Option<RawDisplayHandle>,
) -> Option<WindowIdentifier> {
    window.map(|window| {
        block_on(Box::pin(async move {
            WindowIdentifier::from_raw_handle(&window, display.as_ref()).await
        }))
    })?
}

impl From<&Filter> for FileFilter {
    fn from(filter: &Filter) -> Self {
        let mut ashpd_filter = FileFilter::new(&filter.name);
        for file_extension in &filter.extensions {
            if file_extension == "*" || file_extension.is_empty() {
                ashpd_filter = ashpd_filter.glob("*");
            } else {
                ashpd_filter = ashpd_filter.glob(&format!("*.{file_extension}"));
            }
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
            let res = OpenFileRequest::send(
                to_window_identifier(self.parent, self.parent_display),
                self.title.as_deref().unwrap_or_default(),
                &OpenFileOptions {
                    multiple: Some(false),
                    filters: self.filters.iter().map(Into::into).collect(),
                    current_folder: self
                        .starting_directory
                        .as_ref()
                        .and_then(|f| FilePath::new(f).ok()),
                    ..Default::default()
                },
            );

            if let Err(err) = res {
                error!("Failed to pick file: {err}");
                match zenity::pick_file(&self).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("Failed to pick file with zenity: {err}");
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
            let res = OpenFileRequest::send(
                to_window_identifier(self.parent, self.parent_display),
                self.title.as_deref().unwrap_or_default(),
                &OpenFileOptions {
                    multiple: Some(true),
                    filters: self.filters.iter().map(Into::into).collect(),
                    current_folder: self
                        .starting_directory
                        .as_ref()
                        .and_then(|f| FilePath::new(f).ok()),
                    ..Default::default()
                },
            );

            if let Err(err) = res {
                error!("Failed to pick files: {err}");
                match zenity::pick_files(&self).await {
                    Ok(res) => Some(res.into_iter().map(FileHandle::from).collect::<Vec<_>>()),
                    Err(err) => {
                        error!("Failed to pick files with zenity: {err}");
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
            let res = OpenFileRequest::send(
                to_window_identifier(self.parent, self.parent_display),
                self.title.as_deref().unwrap_or_default(),
                &OpenFileOptions {
                    multiple: Some(false),
                    directory: Some(true),
                    filters: self.filters.iter().map(Into::into).collect(),
                    current_folder: self
                        .starting_directory
                        .as_ref()
                        .and_then(|f| FilePath::new(f).ok()),
                    ..Default::default()
                },
            );

            if let Err(err) = res {
                error!("Failed to pick folder: {err}");
                match zenity::pick_folder(&self).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("Failed to pick folder with zenity: {err}");
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
            let res = OpenFileRequest::send(
                to_window_identifier(self.parent, self.parent_display),
                self.title.as_deref().unwrap_or_default(),
                &OpenFileOptions {
                    multiple: Some(true),
                    directory: Some(true),
                    filters: self.filters.iter().map(Into::into).collect(),
                    current_folder: self
                        .starting_directory
                        .as_ref()
                        .and_then(|f| FilePath::new(f).ok()),
                    ..Default::default()
                },
            );

            if let Err(err) = res {
                error!("Failed to pick folders: {err}");
                match zenity::pick_folders(&self).await {
                    Ok(res) => Some(res.into_iter().map(FileHandle::from).collect::<Vec<_>>()),
                    Err(err) => {
                        error!("Failed to pick folders with zenity: {err}");
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
            let res = SaveFileRequest::send(
                to_window_identifier(self.parent, self.parent_display),
                self.title.as_deref().unwrap_or_default(),
                &SaveFileOptions {
                    filters: self.filters.iter().map(Into::into).collect(),
                    current_folder: self
                        .starting_directory
                        .as_ref()
                        .and_then(|f| FilePath::new(f).ok()),
                    current_name: self.file_name.clone(),
                    ..Default::default()
                },
            );

            if let Err(err) = res {
                error!("Failed to save file: {err}");
                match zenity::save_file(&self).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!("Failed to save file with zenity: {err}");
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
                            error!("Failed to open zenity dialog: {err}");
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
                            error!("Failed to open zenity dialog: {err}");
                            MessageDialogResult::Cancel
                        }
                    }
                }
            }
        })
    }
}
