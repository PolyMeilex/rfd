//
// Windows
//

#[cfg(target_os = "windows")]
mod win_cid;

#[cfg(target_os = "windows")]
pub use win_cid::{pick_file, pick_files, pick_folder, save_file};

//
// Linux
//

#[cfg(target_os = "linux")]
mod gtk3;
#[cfg(target_os = "linux")]
pub use gtk3::{pick_file, pick_files, pick_folder, save_file};

//
// MacOs
//

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{pick_file, pick_files, pick_folder, save_file};

//
// Wasm
//

#[cfg(target_arch = "wasm32")]
pub mod wasm;
// #[cfg(target_arch = "wasm32")]
// pub use wasm::{pick_file, pick_files, pick_folder, save_file};


#[cfg(test)]
mod tests {
    
    #[test]
    #[cfg(not(target_arch = "wasm33"))]
    /// Check if all fns are defined
    fn fn_def_check() {
        #[allow(unused_imports)]
        use super::{pick_file,pick_files,pick_folder,save_file};
    }
}
