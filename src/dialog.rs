use crate::FileHandle;

use std::path::Path;
use std::path::PathBuf;

pub(crate) struct Filter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Default)]
pub struct FileDialog {
    pub(crate) filters: Vec<Filter>,
    pub(crate) starting_directory: Option<PathBuf>,
}

impl FileDialog {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.filters.push(Filter {
            name: name.into(),
            extensions: extensions.iter().map(|e| e.to_string()).collect(),
        });
        self
    }

    pub fn set_directory<P: AsRef<Path>>(mut self, path: &P) -> Self {
        self.starting_directory = Some(path.as_ref().into());
        self
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FileDialog {
    pub fn pick_file(self) -> Option<PathBuf> {
        crate::backend::pick_file(self)
    }

    pub fn pick_files(self) -> Option<Vec<PathBuf>> {
        crate::backend::pick_files(self)
    }

    pub fn pick_folder(self) -> Option<PathBuf> {
        crate::backend::pick_folder(self)
    }

    pub fn save_file(self) -> Option<PathBuf> {
        crate::backend::save_file(self)
    }
}

#[derive(Default)]
pub struct AsyncFileDialog {
    file_dialog: FileDialog,
}

impl AsyncFileDialog {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.file_dialog = self.file_dialog.add_filter(name, extensions);
        self
    }

    pub fn set_directory<P: AsRef<Path>>(mut self, path: &P) -> Self {
        self.file_dialog = self.file_dialog.set_directory(path);
        self
    }
}

use std::future::Future;

impl AsyncFileDialog {
    pub fn pick_file(self) -> impl Future<Output = Option<FileHandle>> {
        crate::backend::pick_file_async(self.file_dialog)
    }

    pub fn pick_files(self) -> impl Future<Output = Option<Vec<FileHandle>>> {
        crate::backend::pick_files_async(self.file_dialog)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn pick_folder(self) -> impl Future<Output = Option<FileHandle>> {
        crate::backend::pick_folder_async(self.file_dialog)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_file(self) -> impl Future<Output = Option<FileHandle>> {
        crate::backend::save_file_async(self.file_dialog)
    }
}
