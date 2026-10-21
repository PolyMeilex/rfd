//! Rusty File Dialogs is a cross platform library for using native file open/save dialogs.
//! It provides both asynchronous and synchronous APIs. Supported platforms:
//!
//!   * Windows
//!   * macOS
//!   * Linux & BSDs (GTK or XDG Desktop Portal)
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
//! On Linux & BSDs, two backends are available, one using the [GTK Rust bindings](https://gtk-rs.org/)
//! and the other using the [XDG Desktop Portal](https://github.com/flatpak/xdg-desktop-portal)
//! D-Bus API through `libdbus` or [zenity](https://gitlab.gnome.org/GNOME/zenity).
//!
//! ## GTK backend
//! The GTK backend is used when the `xdg-portal` feature is disabled with the [`default-features = false`](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features), and `gtk3` or `gtk4` is enabled instead. The GTK
//! backend requires the C library and development headers to be installed to build RFD. The package
//! names on various distributions are:
//!
//! | Distribution    | GTK3                     | GTK4                     |
//! | --------------- | ------------------------ | ------------------------ |
//! | Fedora          | dnf install gtk3-devel   | dnf install gtk4-devel   |
//! | Arch            | pacman -S gtk3           | pacman -S gtk4           |
//! | Debian & Ubuntu | apt install libgtk-3-dev | apt install libgtk-4-dev |
//!
//! ## XDG Desktop Portal backend
//! The XDG Desktop Portal backend is used with the `xdg-portal` Cargo feature which is enabled by default. This backend will use either the GTK or KDE file dialog depending on the desktop environment
//! in use at runtime.
//!
//! It requires the user to have `libdbus` system library (should be available on any systemd
//! distro), if the library is not found `zenity` will be used instead.
//!
//! It requires the user to also have either the
//! [GTK](https://github.com/flatpak/xdg-desktop-portal-gtk),
//! [GNOME](https://gitlab.gnome.org/GNOME/xdg-desktop-portal-gnome), or
//! [KDE](https://invent.kde.org/plasma/xdg-desktop-portal-kde/) XDG Desktop Portal backend installed
//! at runtime. These are typically installed by the distribution together with the desktop environment.
//!
//! The
//! [wlroots portal backend](https://github.com/emersion/xdg-desktop-portal-wlr) does not implement the
//! D-Bus API that RFD requires (it does not interfere with the other portal implementations;
//! they can all be installed simultaneously).
//!
//! [Zenity](https://gitlab.gnome.org/GNOME/zenity) is also required to display message dialogs,
//! and is used for file dialogs if xdg portal fails.
//!
//! If you are packaging an application that uses RFD, ensure that both a supported portal backend and
//! Zenity are installed with the package.
//!
//! # macOS event loops and threading
//!
//! File dialogs execute on the OS main thread. Their calls use the main thread directly or
//! dispatch work to it; they do not take ownership of the application's event loop.
//! A background caller requires the main thread to keep processing dispatch work. Calling from
//! a worker without that processing can wait indefinitely. Do not block the main thread waiting
//! for a worker that may call a dialog.
//!
//! ## Asynchronous file dialogs
//!
//! With a running application loop and a suitable parent window, `AsyncFileDialog` presents a
//! sheet. Prefer an explicit `set_parent`; otherwise rfd looks for an application window.
//! When a sheet cannot be used, it falls back to a synchronous modal dialog on main. Constructing
//! that future can block until the dialog closes, including when called from a worker with a
//! running host or standalone loop. Awaiting the returned future does not make this fallback
//! asynchronous. `AsyncMessageDialog` without a parent uses a separate backend and does not
//! follow this sheet/fallback selection.
//!
//! # Customize button texts of message dialog in Windows
//!
//! `TaskDialogIndirect` API is used for showing message dialog which can have customized button texts.
//! It is only provided by ComCtl32.dll v6 but Windows use v5 by default.
//! If you want to customize button texts or just need a modern dialog style (aka *visual styles*), you will need to:
//!
//! 1. Enable cargo feature `common-controls-v6`.
//! 2. Add an application manifest to use ComCtl32.dll v5. See [Windows Controls / Enabling Visual Styles](https://docs.microsoft.com/en-us/windows/win32/controls/cookbook-overview)
//!
//!
//! Here is an [example](https://github.com/PolyMeilex/rfd/tree/master/examples/message-custom-buttons) using [embed-resource](https://docs.rs/embed-resource/latest/embed_resource/).
//!
//! # Cargo features
//!  * `gtk3`: Uses GTK3 for dialogs on Linux & BSDs; has no effect on Windows and macOS
//!  * `gtk4`: Uses GTK4 for dialogs on Linux & BSDs; has no effect on Windows and macOS
//!  * `xdg-portal`: Uses XDG Desktop Portal instead of GTK on Linux & BSDs
//!  * `common-controls-v6`: Use `TaskDialogIndirect` API from ComCtl32.dll v6 for showing message dialog. This is necessary if you need to customize dialog button texts.
//!
//! # State
//!
//! | API Stability |
//! | ------------- |
//! | 🚧            |
//!
//! | Feature      | Linux | Windows | MacOS     | Wasm32 |
//! | ------------ | ----- | ------- | --------- | ------ |
//! | SingleFile   | ✔     | ✔       | ✔         | ✔      |
//! | MultipleFile | ✔     | ✔       | ✔         | ✔      |
//! | PickFolder   | ✔     | ✔       | ✔         | ✖      |
//! | SaveFile     | ✔     | ✔       | ✔         | ✖      |
//! | Filters      | ✔     | ✔       | ✔         | ✔      |
//! | StartingPath | ✔     | ✔       | ✔         | ✖      |
//! | Async        | ✔     | ✔       | ✔         | ✔      |
//!
//! # rfd-extras
//!
//! AKA features that are not file related
//!
//! | Feature       | Linux        | Windows | MacOS | Wasm32 |
//! | ------------- | -----        | ------- | ----- | ------ |
//! | MessageDialog | ✔            | ✔       | ✔     | ✔      |
//! | PromptDialog  |              |         |       |        |
//! | ColorPicker   |              |         |       |        |

mod backend;

mod file_handle;
pub use file_handle::FileHandle;

mod file_dialog;
mod oneshot;

#[cfg(not(target_family = "wasm"))]
pub use file_dialog::FileDialog;

pub use file_dialog::AsyncFileDialog;

mod message_dialog;
pub use message_dialog::{
    AsyncMessageDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel,
};
