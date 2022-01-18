![img](https://i.imgur.com/YPAgTdf.png)

[![version](https://img.shields.io/crates/v/rfd.svg)](https://crates.io/crates/rfd)
[![Documentation](https://docs.rs/rfd/badge.svg)](https://docs.rs/rfd)
[![dependency status](https://deps.rs/crate/rfd/0.6.4/status.svg)](https://deps.rs/crate/rfd/0.6.4)

Rusty file dialogs for Windows, Linux (GTK), MacOS And WASM32.

# Why RFD?

- It uses 100% native API on all platforms, it does not spawn any processes in the background.
- It supports async/await syntax
- And if one day you decide to port your program to browser, WASM support is there for you!

# Dependencies
#### On Linux:
###### For GTK version:
- GTK3 development libraries (on debian `libgtk-3-dev` on arch `gtk3`)
###### For XFG Portal version (in case you out-out of GTK version):
- XDG Portal provider of you choice has to be present on the system. (Most distros have one by default) 

# Features
- `parent` Adds a dialog parenting support via `raw-window-handle`
- `gtk3` Uses GTK for dialogs, if you know for sure that your users have XDG Portal around you can safely disable this, and drop C dependency 

# Example

```rust
// Sync Dialog
let files = FileDialog::new()
    .add_filter("text", &["txt", "rs"])
    .add_filter("rust", &["rs", "toml"])
    .set_directory("/")
    .pick_file();

// Async Dialog
let file = AsyncFileDialog::new()
    .add_filter("text", &["txt", "rs"])
    .add_filter("rust", &["rs", "toml"])
    .set_directory("/")
    .pick_file()
    .await;

let data = file.read().await;
```

# State

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PolyMeilex/rfd/Rust/master?style=flat-square)

| API Stability |
| ------------- |
| 🚧             |

| Feature      | Linux | Windows | MacOS     | Wasm32 |
| ------------ | ----- | ------- | --------- | ------ |
| SingleFile   | ✔     | ✔       | ✔         | ✔      |
| MultipleFile | ✔     | ✔       | ✔         | ✔      |
| PickFolder   | ✔     | ✔       | ✔         | ✖      |
| SaveFile     | ✔     | ✔       | ✔         | ✖      |
|              |       |         |           |        |
| Filters      | ✔     | ✔       | ✔         | ✔      |
| StartingPath | ✔     | ✔       | ✔         | ✖      |
| Async        | ✔     | ✔       | ✔         | ✔      |

### Difference between `MacOS Windowed App` and `MacOS NonWindowed App`

- Macos async dialog requires a started `NSApplication` instance, so dialog is truly async only when opened in windowed env like `winit`,`SDL2`, etc. otherwise it will fallback to sync dialog.
- It is also recommended to spawn dialogs on main thread, RFD can run dialogs from any thread but it is only possible in windowed app and it adds a little bit of overhead. So it is recommended to: [spawn on main and await in other thread](https://github.com/PolyMeilex/rfd/blob/master/examples/async.rs)
- NonWindowed apps will never be able to spawn dialogs from threads diferent than main
- NonWindowed apps will never be able to spawn async dialogs

# rfd-extras

AKA features that are not file related

| Feature       | Linux | Windows | MacOS | Wasm32 |
| ------------- | ----- | ------- | ----- | ------ |
| MessageDialog | ✔     | ✔       | ✔     | ✔      |
| PromptDialog  |       |         |       |        |
| ColorPicker   |       |         |       |        |
