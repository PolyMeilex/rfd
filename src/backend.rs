#[cfg(target_os = "windows")]
mod win_vista;

#[cfg(target_os = "windows")]
pub use win_vista::{pick_file, pick_files, pick_folder, save_file};

#[cfg(target_os = "linux")]
mod gtk3;
#[cfg(target_os = "linux")]
pub use gtk3::{pick_file, pick_files, pick_folder, save_file};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{pick_file, pick_files, pick_folder, save_file};

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::{pick_file, pick_files, pick_folder, save_file};
