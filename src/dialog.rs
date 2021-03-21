use crate::FileHandle;

use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "parent")]
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub(crate) struct Filter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// ## Synchronous File Dialog
/// #### Supported Platforms:
/// - Linux
/// - Windows
/// - Mac
#[derive(Default)]
pub struct FileDialog {
    pub(crate) filters: Vec<Filter>,
    pub(crate) starting_directory: Option<PathBuf>,
    #[cfg(feature = "parent")]
    pub(crate) parent: Option<RawWindowHandle>,
}

// Oh god, I don't like sending RawWindowHandle between threads but here we go anyways...
// fingers crossed
unsafe impl Send for FileDialog {}

impl FileDialog {
    /// New file dialog builder
    pub fn new() -> Self {
        Default::default()
    }

    /// Add file extension filter.
    ///
    /// Takes in the name of the filter, and list of extensions
    ///
    /// #### Name of the filter will be displayed on supported platforms
    /// - Windows
    /// - Linux
    ///
    /// On platforms that don't support filter names, all filters will be merged into one filter
    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.filters.push(Filter {
            name: name.into(),
            extensions: extensions.iter().map(|e| e.to_string()).collect(),
        });
        self
    }

    /// Set starting directory of the dialog.
    /// #### Supported Platforms:
    /// - Linux
    /// - Windows
    /// - Mac
    pub fn set_directory<P: AsRef<Path>>(mut self, path: &P) -> Self {
        self.starting_directory = Some(path.as_ref().into());
        self
    }

    #[cfg(feature = "parent")]
    fn set_parent<W: HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.parent = Some(parent.raw_window_handle());
        self
    }
}

use crate::backend::{FilePickerDialogImpl, FileSaveDialogImpl, FolderPickerDialogImpl};

#[cfg(not(target_arch = "wasm32"))]
impl FileDialog {
    /// Pick one file
    pub fn pick_file(self) -> Option<PathBuf> {
        FilePickerDialogImpl::pick_file(self)
    }

    /// Pick multiple files
    pub fn pick_files(self) -> Option<Vec<PathBuf>> {
        FilePickerDialogImpl::pick_files(self)
    }

    /// Pick one folder
    pub fn pick_folder(self) -> Option<PathBuf> {
        FolderPickerDialogImpl::pick_folder(self)
    }

    /// Opens save file dialog
    ///
    /// #### Platform specific notes regarding save dialog filters:
    /// - On MacOs
    ///     - If filter is set, all files will be grayed out (no matter the extension sadly)
    ///     - If user does not type an extension MacOs will append first available extension from filters list
    ///     - If user types in filename with extension MacOs will check if it exists in filters list, if not it will display appropriate message
    /// - On GTK
    ///     - It only filters which already existing files get shown to the user
    ///     - It does not append extensions automatically
    ///     - It does not prevent users from adding any unsupported extension
    /// - On Win:
    ///     - If no extension was provided it will just add currently selected one
    ///     - If selected extension was typed in by the user it will just return
    ///     - If unselected extension was provided it will append selected one at the end, example: `test.png.txt`
    pub fn save_file(self) -> Option<PathBuf> {
        FileSaveDialogImpl::save_file(self)
    }
}

/// ## Asynchronous File Dialog
/// #### Supported Platforms:
/// - Linux
/// - Windows
/// - Mac
/// - WASM32
#[derive(Default)]
pub struct AsyncFileDialog {
    file_dialog: FileDialog,
}

impl AsyncFileDialog {
    /// New file dialog builder
    pub fn new() -> Self {
        Default::default()
    }

    /// Add file extension filter.
    ///
    /// Takes in the name of the filter, and list of extensions
    ///
    /// #### Name of the filter will be displayed on supported platforms
    /// - Windows
    /// - Linux
    ///
    /// On platforms that don't support filter names, all filters will be merged into one filter
    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.file_dialog = self.file_dialog.add_filter(name, extensions);
        self
    }

    /// Set starting directory of the dialog.
    /// #### Supported Platforms:
    /// - Linux
    /// - Windows
    /// - Mac
    pub fn set_directory<P: AsRef<Path>>(mut self, path: &P) -> Self {
        self.file_dialog = self.file_dialog.set_directory(path);
        self
    }

    #[cfg(feature = "parent")]
    /// Set parent windows explicitly (optional)
    /// Suported in: `macos`
    pub fn set_parent<W: HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.file_dialog = self.file_dialog.set_parent(parent);
        self
    }
}

use crate::backend::{
    AsyncFilePickerDialogImpl, AsyncFileSaveDialogImpl, AsyncFolderPickerDialogImpl,
};
use std::future::Future;

impl AsyncFileDialog {
    /// Pick one file
    pub fn pick_file(self) -> impl Future<Output = Option<FileHandle>> {
        AsyncFilePickerDialogImpl::pick_file_async(self.file_dialog)
    }

    /// Pick multiple files
    pub fn pick_files(self) -> impl Future<Output = Option<Vec<FileHandle>>> {
        AsyncFilePickerDialogImpl::pick_files_async(self.file_dialog)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Pick one folder
    ///
    /// Does not exist in `WASM32`
    pub fn pick_folder(self) -> impl Future<Output = Option<FileHandle>> {
        AsyncFolderPickerDialogImpl::pick_folder_async(self.file_dialog)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Opens save file dialog
    ///
    /// Does not exist in `WASM32`
    ///
    ///
    /// #### Platform specific notes regarding save dialog filters:
    /// - On MacOs
    ///     - If filter is set, all files will be grayed out (no matter the extension sadly)
    ///     - If user does not type an extension MacOs will append first available extension from filters list
    ///     - If user types in filename with extension MacOs will check if it exists in filters list, if not it will display appropriate message
    /// - On GTK
    ///     - It only filters which already existing files get shown to the user
    ///     - It does not append extensions automatically
    ///     - It does not prevent users from adding any unsupported extension
    /// - On Win:
    ///     - If no extension was provided it will just add currently selected one
    ///     - If selected extension was typed in by the user it will just return
    ///     - If unselected extension was provided it will append selected one at the end, example: `test.png.txt`
    pub fn save_file(self) -> impl Future<Output = Option<FileHandle>> {
        AsyncFileSaveDialogImpl::save_file_async(self.file_dialog)
    }
}

use crate::backend::AsyncMessageDialogImpl;
use crate::backend::MessageDialogImpl;

/// ## Synchronous Message Dialog
#[derive(Default)]
pub struct MessageDialog {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) level: MessageLevel,
    pub(crate) buttons: MessageButtons,
    #[cfg(feature = "parent")]
    pub(crate) parent: Option<RawWindowHandle>,
}

// Oh god, I don't like sending RawWindowHandle between threads but here we go anyways...
// fingers crossed
unsafe impl Send for MessageDialog {}

impl MessageDialog {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_level(mut self, level: MessageLevel) -> Self {
        self.level = level;
        self
    }

    pub fn set_title(mut self, text: &str) -> Self {
        self.title = text.into();
        self
    }

    pub fn set_description(mut self, text: &str) -> Self {
        self.description = text.into();
        self
    }

    pub fn set_buttons(mut self, btn: MessageButtons) -> Self {
        self.buttons = btn;
        self
    }

    #[cfg(feature = "parent")]
    fn set_parent<W: HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.parent = Some(parent.raw_window_handle());
        self
    }

    pub fn show(self) -> bool {
        MessageDialogImpl::show(self)
    }
}

/// ## Asynchronous Message Dialog
#[derive(Default)]
pub struct AsyncMessageDialog(MessageDialog);

impl AsyncMessageDialog {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_level(mut self, level: MessageLevel) -> Self {
        self.0 = self.0.set_level(level);
        self
    }

    pub fn set_title(mut self, text: &str) -> Self {
        self.0 = self.0.set_title(text);
        self
    }

    pub fn set_description(mut self, text: &str) -> Self {
        self.0 = self.0.set_description(text);
        self
    }

    pub fn set_buttons(mut self, btn: MessageButtons) -> Self {
        self.0 = self.0.set_buttons(btn);
        self
    }

    #[cfg(feature = "parent")]
    /// Set parent windows explicitly (optional)
    /// Suported in: `macos`
    pub fn set_parent<W: HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.0 = self.0.set_parent(parent);
        self
    }

    pub fn show(self) -> impl Future<Output = bool> {
        AsyncMessageDialogImpl::show_async(self.0)
    }
}

pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

impl Default for MessageLevel {
    fn default() -> Self {
        Self::Info
    }
}

pub enum MessageButtons {
    Ok,
    OkCancle,
    YesNo,
}

impl Default for MessageButtons {
    fn default() -> Self {
        Self::Ok
    }
}
