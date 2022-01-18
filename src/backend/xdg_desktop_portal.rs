use std::path::PathBuf;

use crate::backend::DialogFutureType;
use crate::file_dialog::Filter;
use crate::{FileDialog, FileHandle};

use ashpd::desktop::file_chooser::{
    FileChooserProxy, FileFilter, OpenFileOptions, SaveFileOptions,
};
// TODO: convert raw_window_handle::RawWindowHandle to ashpd::WindowIdentifier
// https://github.com/bilelmoussaoui/ashpd/issues/40
use ashpd::{zbus, WindowIdentifier};

use log::warn;
use pollster::block_on;

//
// Utility functions
//

fn add_filters_to_open_file_options(
    filters: Vec<Filter>,
    mut options: OpenFileOptions,
) -> OpenFileOptions {
    for filter in &filters {
        let mut ashpd_filter = FileFilter::new(&filter.name);
        for file_extension in &filter.extensions {
            ashpd_filter = ashpd_filter.glob(&format!("*.{}", file_extension));
        }
        options = options.add_filter(ashpd_filter);
    }
    options
}

fn add_filters_to_save_file_options(
    filters: Vec<Filter>,
    mut options: SaveFileOptions,
) -> SaveFileOptions {
    for filter in &filters {
        let mut ashpd_filter = FileFilter::new(&filter.name);
        for file_extension in &filter.extensions {
            ashpd_filter = ashpd_filter.glob(&format!("*.{}", file_extension));
        }
        options = options.add_filter(ashpd_filter);
    }
    options
}

// refer to https://github.com/flatpak/xdg-desktop-portal/issues/213
fn uri_to_pathbuf(uri: &str) -> Option<PathBuf> {
    uri.strip_prefix("file://").map(PathBuf::from)
}

fn ok_or_warn<T, E: std::fmt::Debug>(result: Result<T, E>) -> Option<T> {
    match result {
        Err(e) => {
            warn!("{:?}", e);
            None
        }
        Ok(t) => Some(t),
    }
}

async fn file_chooser_proxy<'a>() -> Option<FileChooserProxy<'a>> {
    let connection = ok_or_warn(zbus::Connection::session().await)?;
    ok_or_warn(FileChooserProxy::new(&connection).await)
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
        Box::pin(async {
            let proxy = file_chooser_proxy().await?;
            let mut options = OpenFileOptions::default()
                .accept_label("Pick file")
                .multiple(false);
            options = add_filters_to_open_file_options(self.filters, options);
            let selected_files = proxy
                .open_file(
                    &WindowIdentifier::default(),
                    &self.title.unwrap_or_else(|| "Pick a file".to_string()),
                    options,
                )
                .await;
            if selected_files.is_err() {
                return None;
            }
            uri_to_pathbuf(&selected_files.unwrap().uris()[0]).map(FileHandle::from)
        })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        Box::pin(async {
            let proxy = file_chooser_proxy().await?;
            let mut options = OpenFileOptions::default()
                .accept_label("Pick file(s)")
                .multiple(true);
            options = add_filters_to_open_file_options(self.filters, options);
            let selected_files = proxy
                .open_file(
                    &WindowIdentifier::default(),
                    &self
                        .title
                        .unwrap_or_else(|| "Pick one or more files".to_string()),
                    options,
                )
                .await;
            if selected_files.is_err() {
                return None;
            }
            let selected_files = selected_files
                .unwrap()
                .uris()
                .iter()
                .filter_map(|string| uri_to_pathbuf(string))
                .map(FileHandle::from)
                .collect::<Vec<FileHandle>>();
            if selected_files.is_empty() {
                return None;
            }
            Some(selected_files)
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
}

use crate::backend::AsyncFolderPickerDialogImpl;
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        Box::pin(async {
            let proxy = file_chooser_proxy().await?;
            let mut options = OpenFileOptions::default()
                .accept_label("Pick folder")
                .multiple(false)
                .directory(true);
            options = add_filters_to_open_file_options(self.filters, options);
            let selected_files = proxy
                .open_file(
                    &WindowIdentifier::default(),
                    &self.title.unwrap_or_else(|| "Pick a folder".to_string()),
                    options,
                )
                .await;
            if selected_files.is_err() {
                return None;
            }
            uri_to_pathbuf(&selected_files.unwrap().uris()[0]).map(FileHandle::from)
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
        Box::pin(async {
            let proxy = file_chooser_proxy().await?;
            let mut options = SaveFileOptions::default().accept_label("Save");
            options = add_filters_to_save_file_options(self.filters, options);
            if let Some(file_name) = self.file_name {
                options = options.current_name(&file_name);
            }
            // TODO: impl zvariant::Type for PathBuf?
            // if let Some(dir) = self.starting_directory {
            //    options.current_folder(dir);
            // }
            let selected_files = proxy
                .save_file(
                    &WindowIdentifier::default(),
                    &self.title.unwrap_or_else(|| "Save file".to_string()),
                    options,
                )
                .await;
            if selected_files.is_err() {
                return None;
            }
            uri_to_pathbuf(&selected_files.unwrap().uris()[0]).map(FileHandle::from)
        })
    }
}
