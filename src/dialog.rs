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
    dialog_type: DialogType,
    filters: Vec<Filter<'a>>,
    starting_directory: Option<&'a Path>,
}

impl<'a> Dialog<'a> {
    pub fn new(dialog_type: DialogType) -> Self {
        Self {
            dialog_type,
            ..Default::default()
        }
    }

    pub fn pick_file() -> Self {
        Self::new(DialogType::PickFile)
    }

    pub fn pick_files() -> Self {
        Self::new(DialogType::PickFiles)
    }

    pub fn pick_folder() -> Self {
        Self::new(DialogType::PickFolder)
    }

    pub fn save_file() -> Self {
        Self::new(DialogType::SaveFile)
    }

    pub fn filter(mut self, name: &'a str, extensions: &'a [&'a str]) -> Self {
        self.filters.push(Filter { name, extensions });
        self
    }

    pub fn starting_directory<P: AsRef<Path>>(mut self, path: &'a P) -> Self {
        self.starting_directory = Some(path.as_ref());
        self
    }

    pub fn open(self) -> Response {
        let opt = DialogOptions {
            filters: &self.filters,
            starting_directory: self.starting_directory,
        };
        match self.dialog_type {
            DialogType::PickFile => crate::pick_file(opt)
                .map(|f| Response::Single(f))
                .unwrap_or(Response::None),
            DialogType::PickFiles => crate::pick_files(opt)
                .map(|f| Response::Multiple(f))
                .unwrap_or(Response::None),
            DialogType::PickFolder => crate::pick_folder(opt)
                .map(|f| Response::Single(f))
                .unwrap_or(Response::None),
            DialogType::SaveFile => crate::save_file(opt)
                .map(|f| Response::Single(f))
                .unwrap_or(Response::None),
        }
    }
}

pub enum Response {
    Single(PathBuf),
    Multiple(Vec<PathBuf>),
    None,
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
