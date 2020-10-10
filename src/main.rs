#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod gtk3;

mod file_dialog {
    #[cfg(windows)]
    pub use crate::windows::open;

    #[cfg(target_os = "linux")]
    pub use crate::gtk3::open;

    #[cfg(target_os = "macos")]
    pub use crate::macos::open;
}

pub struct DialogParams<'a> {
    filters: Vec<(&'a str, &'a str)>,
}

fn main() {
    let params = DialogParams {
        filters: vec![(".mid", "*.mid"), (".midi", "*.midi")],
    };

    let path = file_dialog::open(params);

    println!("{:?}", path);
}
