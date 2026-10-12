use std::path::Path;
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::PathBuf};

mod portal;

mod window_identifier;
use window_identifier::WindowIdentifier;

use super::linux::zenity;
use crate::backend::DialogFutureType;
use crate::file_dialog::Filter;
use crate::message_dialog::MessageDialog;
use crate::{FileDialog, FileHandle, MessageButtons, MessageDialogResult};

use log::{error, warn};
use pollster::block_on;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

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

impl From<&Filter> for portal::FileFilter {
    fn from(filter: &Filter) -> Self {
        let name = CString::new(filter.name.as_str()).unwrap();

        let globs = filter
            .extensions
            .iter()
            .map(|file_extension| {
                if file_extension == "*" || file_extension.is_empty() {
                    c"*".to_owned()
                } else {
                    CString::new(format!("*.{file_extension}")).unwrap()
                }
            })
            .collect();

        (name, globs)
    }
}

fn str_to_cstring(value: Option<&str>) -> Option<CString> {
    value.and_then(|title| CString::new(title).ok())
}

fn path_to_cstring(value: Option<&Path>) -> Option<portal::FilePath> {
    value
        .and_then(|f| CString::new(f.as_os_str().as_bytes()).ok())
        .map(portal::FilePath)
}

//
// File Picker
//

use crate::backend::FilePickerDialogImpl;
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        let window_identifier = to_window_identifier(self.parent, self.parent_display);
        let res = portal::open_file(portal::OpenFileOptions {
            parent_window: window_identifier
                .as_ref()
                .and_then(|w| CString::new(w.to_string()).ok())
                .unwrap_or_default(),
            title: str_to_cstring(self.title.as_deref()).unwrap_or_default(),
            multiple: Some(false),
            filters: self.filters.iter().map(Into::into).collect(),
            current_folder: path_to_cstring(self.starting_directory.as_deref()),
            ..Default::default()
        });

        if let Some(res) = res {
            match res {
                portal::Response::Selected(res, _) if res.is_empty() => None,
                portal::Response::Selected(mut res, _) => Some(res.remove(0)),
                portal::Response::Canceled => None,
            }
        } else {
            warn!("Using zenity fallback");
            match block_on(zenity::pick_file(&self)) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to pick file with zenity: {err}");
                    None
                }
            }
        }
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        let window_identifier = to_window_identifier(self.parent, self.parent_display);
        let res = portal::open_file(portal::OpenFileOptions {
            parent_window: window_identifier
                .as_ref()
                .and_then(|w| CString::new(w.to_string()).ok())
                .unwrap_or_default(),
            title: str_to_cstring(self.title.as_deref()).unwrap_or_default(),
            multiple: Some(true),
            filters: self.filters.iter().map(Into::into).collect(),
            current_folder: path_to_cstring(self.starting_directory.as_deref()),
            ..Default::default()
        });

        if let Some(res) = res {
            match res {
                portal::Response::Selected(res, _) => Some(res),
                portal::Response::Canceled => None,
            }
        } else {
            warn!("Using zenity fallback");
            match block_on(zenity::pick_files(&self)) {
                Ok(res) => Some(res),
                Err(err) => {
                    error!("Failed to pick files with zenity: {err}");
                    None
                }
            }
        }
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
        let window_identifier = to_window_identifier(self.parent, self.parent_display);
        let res = portal::open_file(portal::OpenFileOptions {
            parent_window: window_identifier
                .as_ref()
                .and_then(|w| CString::new(w.to_string()).ok())
                .unwrap_or_default(),
            title: str_to_cstring(self.title.as_deref()).unwrap_or_default(),
            multiple: Some(false),
            directory: Some(true),
            filters: self.filters.iter().map(Into::into).collect(),
            current_folder: path_to_cstring(self.starting_directory.as_deref()),
            ..Default::default()
        });

        if let Some(res) = res {
            match res {
                portal::Response::Selected(res, _) if res.is_empty() => None,
                portal::Response::Selected(mut res, _) => Some(res.remove(0)),
                portal::Response::Canceled => None,
            }
        } else {
            warn!("Using zenity fallback");
            match block_on(zenity::pick_folder(&self)) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to pick folder with zenity: {err}");
                    None
                }
            }
        }
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        let window_identifier = to_window_identifier(self.parent, self.parent_display);
        let res = portal::open_file(portal::OpenFileOptions {
            parent_window: window_identifier
                .as_ref()
                .and_then(|w| CString::new(w.to_string()).ok())
                .unwrap_or_default(),
            title: str_to_cstring(self.title.as_deref()).unwrap_or_default(),
            multiple: Some(true),
            directory: Some(true),
            filters: self.filters.iter().map(Into::into).collect(),
            current_folder: path_to_cstring(self.starting_directory.as_deref()),
            ..Default::default()
        });

        if let Some(res) = res {
            match res {
                portal::Response::Selected(res, _) => Some(res),
                portal::Response::Canceled => None,
            }
        } else {
            warn!("Using zenity fallback");
            match block_on(zenity::pick_folders(&self)) {
                Ok(res) => Some(res),
                Err(err) => {
                    error!("Failed to pick folders with zenity: {err}");
                    None
                }
            }
        }
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

use crate::backend::FileSaveDialogImpl;
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        let window_identifier = to_window_identifier(self.parent, self.parent_display);
        let res = portal::save_file(portal::SaveFileOptions {
            parent_window: window_identifier
                .as_ref()
                .and_then(|w| CString::new(w.to_string()).ok())
                .unwrap_or_default(),
            title: str_to_cstring(self.title.as_deref()).unwrap_or_default(),
            filters: self.filters.iter().map(Into::into).collect(),
            current_folder: path_to_cstring(self.starting_directory.as_deref()),
            current_name: str_to_cstring(self.file_name.as_deref()),
            ..Default::default()
        });

        if let Some(res) = res {
            match res {
                portal::Response::Selected(res, _) if res.is_empty() => None,
                portal::Response::Selected(mut res, current_filter) => {
                    let path = res.remove(0);
                    let chosen = current_filter.and_then(|label| {
                        self.filters
                            .iter()
                            .find(|filter| filter.name.as_bytes() == label.as_bytes())
                    });
                    match chosen.and_then(|filter| {
                        with_chosen_extension(&path, self.file_name.as_deref(), filter)
                    }) {
                        // The portal only asked to overwrite the path it returned
                        Some(renamed) if renamed.exists() => {
                            warn!("Not renaming {path:?} to {renamed:?}: it already exists");
                            Some(path)
                        }
                        Some(renamed) => Some(renamed),
                        None => Some(path),
                    }
                }
                portal::Response::Canceled => None,
            }
        } else {
            warn!("Using zenity fallback");
            match block_on(zenity::save_file(&self)) {
                Ok(res) => res,
                Err(err) => {
                    error!("Failed to save file with zenity: {err}");
                    None
                }
            }
        }
    }
}

/// The path with the extension of the filter that was selected in the save dialog, when the
/// portal didn't apply it (GTK keeps the name as typed when a different filter is selected).
///
/// An extension the user typed is kept: the path is only changed when it has no extension, or
/// still has the extension of `file_name` (the name the dialog was opened with) and that
/// extension isn't one of the filter's.
fn with_chosen_extension(path: &Path, file_name: Option<&str>, filter: &Filter) -> Option<PathBuf> {
    let new_ext = filter
        .extensions
        .first()
        .filter(|ext| !ext.is_empty() && *ext != "*")?;
    let Some(ext) = path.extension() else {
        return Some(path.with_extension(new_ext));
    };
    let ext = ext.to_string_lossy();
    if filter
        .extensions
        .iter()
        .any(|e| e.eq_ignore_ascii_case(&ext))
    {
        return None;
    }
    let suggested = file_name.and_then(|name| Path::new(name).extension())?;
    suggested
        .to_string_lossy()
        .eq_ignore_ascii_case(&ext)
        .then(|| path.with_extension(new_ext))
}

use crate::backend::AsyncFileSaveDialogImpl;
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        async_thread(move || Self::save_file(self).map(FileHandle::wrap))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(name: &str, extensions: &[&str]) -> Filter {
        Filter {
            name: name.into(),
            extensions: extensions.iter().map(|e| e.to_string()).collect(),
        }
    }

    fn renamed(path: &str, file_name: Option<&str>, filter: &Filter) -> Option<PathBuf> {
        with_chosen_extension(Path::new(path), file_name, filter)
    }

    #[test]
    fn chosen_filter_replaces_the_suggested_extension() {
        let png = filter("PNG", &["png"]);
        assert_eq!(
            renamed("/a/Untitled.psd", Some("Untitled.psd"), &png),
            Some(PathBuf::from("/a/Untitled.png"))
        );
        // The user renamed the file but kept the suggested extension
        assert_eq!(
            renamed("/a/cat.psd", Some("Untitled.psd"), &png),
            Some(PathBuf::from("/a/cat.png"))
        );
        assert_eq!(
            renamed("/a/Untitled.PSD", Some("Untitled.psd"), &png),
            Some(PathBuf::from("/a/Untitled.png"))
        );
        assert_eq!(
            renamed("/a/x.png.psd", Some("x.png.psd"), &png),
            Some(PathBuf::from("/a/x.png.png"))
        );
    }

    #[test]
    fn a_typed_extension_is_kept() {
        let psd = filter("PSD", &["psd", "psb"]);
        assert_eq!(renamed("/a/Untitled.png", Some("Untitled.psd"), &psd), None);
        // Without a suggested name there is nothing to tell a typed extension from
        assert_eq!(renamed("/a/Untitled.png", None, &psd), None);
    }

    #[test]
    fn a_matching_extension_is_kept() {
        let psd = filter("PSD", &["psd", "psb"]);
        assert_eq!(renamed("/a/Untitled.psd", Some("Untitled.psd"), &psd), None);
        assert_eq!(renamed("/a/Untitled.psb", Some("Untitled.psd"), &psd), None);
        assert_eq!(renamed("/a/Untitled.PSB", Some("Untitled.psd"), &psd), None);
    }

    #[test]
    fn a_missing_extension_is_added() {
        let psd = filter("PSD", &["psd", "psb"]);
        assert_eq!(
            renamed("/a/Untitled", Some("Untitled.psd"), &psd),
            Some(PathBuf::from("/a/Untitled.psd"))
        );
        assert_eq!(
            renamed("/a/Untitled", None, &psd),
            Some(PathBuf::from("/a/Untitled.psd"))
        );
    }

    #[test]
    fn filters_without_an_extension_change_nothing() {
        for any in [
            filter("All", &["*"]),
            filter("All", &[""]),
            filter("None", &[]),
        ] {
            assert_eq!(renamed("/a/Untitled.psd", Some("Untitled.psd"), &any), None);
            assert_eq!(renamed("/a/Untitled", None, &any), None);
        }
    }
}
