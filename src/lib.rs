#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod gtk3;

pub mod file_dialog {
    #[cfg(windows)]
    pub use crate::windows::open;

    #[cfg(target_os = "linux")]
    pub use crate::gtk3::open;

    #[cfg(target_os = "macos")]
    pub use crate::macos::open;

    #[derive(Default)]
    pub struct DialogParams<'a> {
        pub filters: &'a [(&'a str, &'a str)],
    }

    impl<'a> DialogParams<'a> {
        pub fn new(filters: &'a [(&'a str, &'a str)]) -> Self {
            Self {
                filters,
            }
        }
    }
}
