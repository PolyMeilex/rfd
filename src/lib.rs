mod backend;

mod file_handle;
pub use file_handle::FileHandle;

mod file_dialog;

#[cfg(not(target_arch = "wasm32"))]
pub use file_dialog::FileDialog;

pub use file_dialog::AsyncFileDialog;

#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_family = "wasm",
    all(
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk3"
    )
))]
mod message_dialog;

#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_family = "wasm",
    all(
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd"
        ),
        feature = "gtk3"
    )
))]
pub use message_dialog::{AsyncMessageDialog, MessageButtons, MessageDialog, MessageLevel};
