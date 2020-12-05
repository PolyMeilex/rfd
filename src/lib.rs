use std::path::{Path, PathBuf};

// Old Windows API
// #[cfg(target_os = "windows")]
// mod win_xp;
// #[cfg(target_os = "windows")]
// pub use win_xp::open_multiple_files_with_params;

// New Windows API
#[cfg(target_os = "windows")]
mod win_vista;

#[cfg(target_os = "windows")]
pub use win_vista::{
    open_file_with_params, open_multiple_files_with_params, pick_folder_with_params,
    save_file_with_params,
};

#[cfg(target_os = "linux")]
mod gtk3;
#[cfg(target_os = "linux")]
pub use gtk3::{
    open_file_with_params, open_multiple_files_with_params, pick_folder_with_params,
    save_file_with_params,
};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{
    open_file_with_params, open_multiple_files_with_params, pick_folder_with_params,
    save_file_with_params,
};

pub fn open() -> Option<std::path::PathBuf> {
    open_file_with_params(DialogParams::new())
}

pub fn open_multiple() -> Option<Vec<std::path::PathBuf>> {
    open_multiple_files_with_params(DialogParams::new())
}

pub fn pick_folder() -> Option<PathBuf> {
    pick_folder_with_params(DialogParams::new())
}

/// Paramaters to pass to the file dialog.
#[derive(Default)]
pub struct DialogParams<'a> {
    pub filters: &'a [(&'a str, &'a str)],
    pub starting_directory: Option<PathBuf>,
}

impl<'a> DialogParams<'a> {
    /// Creates a new `DialogParams` with nothing configured.
    pub fn new() -> Self {
        Self {
            filters: &[],
            starting_directory: None,
        }
    }

    /// Sets the filters of this `DialogParams`.
    pub fn set_filters(mut self, filters: &'a [(&'a str, &'a str)]) -> Self {
        self.filters = filters;
        self
    }

    pub fn set_starting_directory<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.starting_directory = Some(path.into());
        self
    }
}
