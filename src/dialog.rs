use std::path::Path;
use std::path::PathBuf;

pub enum DialogType {
    PickFile,
    PickFiles,
    PickFolder,
    SaveFile,
}
impl Default for DialogType {
    fn default() -> Self {
        Self::PickFile
    }
}

pub struct Filter<'a> {
    pub name: &'a str,
    pub extensions: &'a [&'a str],
}

#[derive(Default)]
pub struct Dialog<'a> {
    filters: Vec<Filter<'a>>,
    starting_directory: Option<&'a Path>,
}

impl<'a> Dialog<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_filter(mut self, name: &'a str, extensions: &'a [&'a str]) -> Self {
        self.filters.push(Filter { name, extensions });
        self
    }

    pub fn set_directory<P: AsRef<Path>>(mut self, path: &'a P) -> Self {
        self.starting_directory = Some(path.as_ref());
        self
    }

    pub fn pick_file(&self) -> Option<PathBuf> {
        let opt = DialogOptions {
            filters: &self.filters,
            starting_directory: self.starting_directory,
        };
        crate::pick_file(opt)
    }

    pub fn pick_files(&self) -> Option<Vec<PathBuf>> {
        let opt = DialogOptions {
            filters: &self.filters,
            starting_directory: self.starting_directory,
        };
        crate::pick_files(opt)
    }

    pub fn pick_folder(&self) -> Option<PathBuf> {
        let opt = DialogOptions {
            filters: &self.filters,
            starting_directory: self.starting_directory,
        };
        crate::pick_folder(opt)
    }

    pub fn save_file(&self) -> Option<PathBuf> {
        let opt = DialogOptions {
            filters: &self.filters,
            starting_directory: self.starting_directory,
        };
        crate::save_file(opt)
    }
}

/// Paramaters to pass to the file dialog.
#[derive(Default)]
pub struct DialogOptions<'a> {
    pub filters: &'a [Filter<'a>],
    pub starting_directory: Option<&'a Path>,
}

impl<'a> DialogOptions<'a> {
    /// Creates a new `DialogParams` with nothing configured.
    pub fn new() -> Self {
        Self {
            filters: &[],
            starting_directory: None,
        }
    }

    /// Sets the filters of this `DialogParams`.
    pub fn set_filters(mut self, filters: &'a [Filter<'a>]) -> Self {
        self.filters = filters;
        self
    }

    pub fn set_starting_directory<T: AsRef<Path>>(mut self, path: &'a T) -> Self {
        self.starting_directory = Some(path.as_ref());
        self
    }
}
