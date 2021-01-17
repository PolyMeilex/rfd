# rfd

[![version](https://img.shields.io/crates/v/rfd.svg)](https://crates.io/crates/rfd)
[![Documentation](https://docs.rs/rfd/badge.svg)](https://docs.rs/rfd)
[![dependency status](https://deps.rs/crate/rfd/0.0.3/status.svg)](https://deps.rs/crate/rfd/0.0.3)

WIP native file dialogs for Windows, Linux (GTK), MacOS.

# Example

```rust
// Sync Dialog
let files = FileDialog::new()
    .add_filter("text", &["txt", "rs"])
    .add_filter("rust", &["rs", "toml"])
    .set_directory(&"/")
    .pick_files();

// Async Dialog
let file = AsyncFileDialog::new()
    .add_filter("text", &["txt", "rs"])
    .add_filter("rust", &["rs", "toml"])
    .set_directory(&"/")
    .pick_file()
    .await;

let data = file.read().await;
```

# State

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PolyMeilex/rfd/Rust/master?style=flat-square)

| API Stability |
| ------------- |
| 🚧            |

| Feature      | Linux | Windows | MacOS [1] | Wasm32 |
| ------------ | ----- | ------- | --------- | ------ |
| SingleFile   | ✔     | ✔       | ✔         | ✔      |
| MultipleFile | ✔     | ✔       | ✔         | ✔      |
| PickFolder   | ✔     | ✔       | ✔         | ✖      |
| SaveFile     | ✔     | ✔       | ✔         | ✖      |
|              |       |         |           |        |
| Filters      | ✔     | ✔       | ✔         | ✔      |
| StartingPath | ✔     | ✔       | ✔         | ✖      |
| Async        | ✔     | ✔       | ✔         | ✔      |

[1] Sync dialog freezes when used with winit (same way as `nfd`) [#1779](https://github.com/rust-windowing/winit/issues/1779)

# rfd-extras

AKA features that will be either in a separate `rfd-extras` crate, or behind a feature flag

| Feature       | Linux | Windows | MacOS |
| ------------- | ----- | ------- | ----- |
| MessageDialog |       |         |       |
| PromptDialog  |       |         |       |
| ColorPicker   |       |         |       |
