use std::path::Path;
use std::path::PathBuf;

pub struct Filter<'a> {
    pub name: &'a str,
    pub extensions: &'a [&'a str],
}

#[derive(Default)]
pub struct FileDialog<'a> {
    pub(crate) filters: Vec<Filter<'a>>,
    pub(crate) starting_directory: Option<&'a Path>,
}

impl<'a> FileDialog<'a> {
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
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> FileDialog<'a> {
    pub fn pick_file(&self) -> Option<PathBuf> {
        crate::backend::pick_file(self)
    }

    pub fn pick_files(&self) -> Option<Vec<PathBuf>> {
        crate::backend::pick_files(self)
    }

    pub fn pick_folder(&self) -> Option<PathBuf> {
        crate::backend::pick_folder(self)
    }

    pub fn save_file(&self) -> Option<PathBuf> {
        crate::backend::save_file(self)
    }
}
