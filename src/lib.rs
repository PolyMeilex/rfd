//! Rusty File Dialogs is a cross platform library for using native file open/save dialogs.
//! It provides both asynchronous and synchronous APIs. Supported platforms:
//!
//!   * Windows
//!   * macOS
//!   * Linux & BSDs (GTK3 or XDG Desktop Portal)
//!   * WASM32 (async only)
//!
//! # Examples
//!
//! ## Synchronous
//! ```no_run
//! use rfd::FileDialog;
//!
//! let files = FileDialog::new()
//!     .add_filter("text", &["txt", "rs"])
//!     .add_filter("rust", &["rs", "toml"])
//!     .set_directory("/")
//!     .pick_file();
//! ```
//!
//! ## Asynchronous
//! ```no_run
//! use rfd::AsyncFileDialog;
//!
//! let future = async {
//!     let file = AsyncFileDialog::new()
//!         .add_filter("text", &["txt", "rs"])
//!         .add_filter("rust", &["rs", "toml"])
//!         .set_directory("/")
//!         .pick_file()
//!         .await;
//!
//!     let data = file.unwrap().read().await;
//! };
//! ```
//!
//! # Linux & BSD backends
//!
//! On Linux & BSDs, two backends are available, one using the [GTK3 Rust bindings](https://gtk-rs.org/)
//! and the other using the [XDG Desktop Portal](https://github.com/flatpak/xdg-desktop-portal)
//! D-Bus API through [ashpd](https://github.com/bilelmoussaoui/ashpd) &
//! [zbus](https://gitlab.freedesktop.org/dbus/zbus/).
//!
//! ## GTK backend
//! The GTK backend is used with the `gtk3` Cargo feature which is enabled by default. The GTK3
//! backend requires the C library and development headers to be installed to build RFD. The package
//! names on various distributions are:
//!
//! | Distribution    | Package name |
//! | --------------- | ------------ |
//! | Fedora          | gtk3-devel   |
//! | Arch            | gtk3         |
//! | Debian & Ubuntu | libgtk-3-dev |
//!
//! ## XDG Desktop Portal backend
//! The XDG Desktop Portal backend is used when the `gtk3` feature is disabled with
//! [`default-features = false`](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features). This backend will use either the GTK or KDE file dialog depending on the desktop environment
//! in use at runtime. It does not have any non-Rust
//! build dependencies, however it requires the user to have either the
//! [GTK](https://github.com/flatpak/xdg-desktop-portal-gtk),
//! [GNOME](https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome), or
//! [KDE](https://invent.kde.org/plasma/xdg-desktop-portal-kde/) XDG Desktop Portal backend installed
//! at runtime. These are typically installed by the distribution together with the desktop environment.
//! If you are packaging an application that uses RFD, ensure either one of these is installed
//! with the package. The
//! [wlroots portal backend](https://github.com/emersion/xdg-desktop-portal-wlr) does not implement the
//! D-Bus API that RFD requires (it does not interfere with the other portal implementations;
//! they can all be installed simultaneously).
//!
//! The XDG Desktop Portal has no API for message dialogs, so the [MessageDialog] and
//! [AsyncMessageDialog] structs will not build with this backend.
//!
//! # macOS non-windowed applications, async, and threading
//!
//! macOS async dialogs require an `NSApplication` instance, so the dialog is only truly async when
//! opened in windowed environment like `winit` or `SDL2`. Otherwise, it will fallback to sync dialog.
//! It is also recommended to spawn dialogs on your main thread. RFD can run dialogs from any thread
//! but it is only possible in a windowed app and it adds a little bit of overhead. So it is recommended
//! to [spawn on main and await in other thread](https://github.com/PolyMeilex/rfd/blob/master/examples/async.rs).
//! Non-windowed apps will never be able to spawn async dialogs or from threads other than the main thread.
//!
//! # Cargo features
//!  * `parent`: Adds a dialog parenting support via [raw-window-handle](https://github.com/rust-windowing/raw-window-handle).
//!    [Not yet implemented](https://github.com/bilelmoussaoui/ashpd/issues/40) for XDG Desktop Portal
//!    backend.
//!  * `gtk3`: Uses GTK for dialogs on Linux & BSDs; has no effect on Windows and macOS
//!
//! # State
//!
//! | API Stability |
//! | ------------- |
//! | 🚧             |
//!
//! | Feature      | Linux | Windows | MacOS     | Wasm32 |
//! | ------------ | ----- | ------- | --------- | ------ |
//! | SingleFile   | ✔     | ✔       | ✔         | ✔      |
//! | MultipleFile | ✔     | ✔       | ✔         | ✔      |
//! | PickFolder   | ✔     | ✔       | ✔         | ✖      |
//! | SaveFile     | ✔     | ✔       | ✔         | ✖      |
//! |              |       |         |           |        |
//! | Filters      | ✔ ([GTK only](https://github.com/PolyMeilex/rfd/issues/42)) | ✔ | ✔ | ✔ |
//! | StartingPath | ✔     | ✔       | ✔         | ✖      |
//! | Async        | ✔     | ✔       | ✔         | ✔      |
//!
//! # rfd-extras
//!
//! AKA features that are not file related
//!
//! | Feature       | Linux        | Windows | MacOS | Wasm32 |
//! | ------------- | -----        | ------- | ----- | ------ |
//! | MessageDialog | ✔ (GTK only) | ✔       | ✔     | ✔      |
//! | PromptDialog  |              |         |       |        |
//! | ColorPicker   |              |         |       |        |

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
