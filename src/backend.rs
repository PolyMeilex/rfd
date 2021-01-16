//
// Windows
//

#[cfg(target_os = "windows")]
mod win_cid;
#[cfg(target_os = "windows")]
pub use win_cid::{pick_file, pick_files, pick_folder, save_file};
#[cfg(target_os = "windows")]
pub use win_cid::{pick_file_async, pick_files_async, pick_folder_async, save_file_async};

//
// Linux
//

#[cfg(target_os = "linux")]
mod gtk3;
#[cfg(target_os = "linux")]
pub use gtk3::{pick_file, pick_files, pick_folder, save_file};
#[cfg(target_os = "linux")]
pub use gtk3::{pick_file_async, pick_files_async, pick_folder_async, save_file_async};

//
// MacOs
//

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{pick_file, pick_files, pick_folder, save_file};
#[cfg(target_os = "macos")]
pub use macos::{pick_file_async, pick_files_async, pick_folder_async, save_file_async};

//
// Wasm
//

#[cfg(target_arch = "wasm32")]
pub mod wasm;
// #[cfg(target_arch = "wasm32")]
// pub use wasm::{pick_file, pick_files, pick_folder, save_file};
#[cfg(target_arch = "wasm32")]
pub use wasm::{pick_file_async, pick_files_async, pick_folder_async, save_file_async};

#[cfg(test)]
mod tests {
    #[test]
    /// Check if all fns are defined
    fn fn_def_check() {
        #[allow(unused_imports)]
        #[cfg(not(target_arch = "wasm32"))]
        use super::{pick_file, pick_files, pick_folder, save_file};

        #[allow(unused_imports)]
        use super::{pick_file_async, pick_files_async, pick_folder_async, save_file_async};
    }
}
