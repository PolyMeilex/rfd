# rfd

Very WIP and untested native file dialogs for Windows, Linux (GTK), MacOS.

# Examples

All examples are located in `examples` directory.

- Run `cargo run --example pick_file` for the simple example.
- Run `cargo run --example pick_file_filter` for an example utilizing a filter.
- Run `cargo run --example pick_folder`.
- Run `cargo run --example save`.

# State

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/PolyMeilex/rfd/Rust/master?style=flat-square)

| API Stability               |
| --------------------------- |
| :x: API is not designed yet |

| Feature      | Linux              | Windows                               | MacOS [1]          |
| ------------ | ------------------ | ------------------------------------- | ------------------ |
| SingleFile   | :heavy_check_mark: | :white_check_mark: :heavy_check_mark: | :heavy_check_mark: |
| MultipleFile | :heavy_check_mark: | :white_check_mark: :heavy_check_mark: | :heavy_check_mark: |
| PickFolder   | :heavy_check_mark: | :heavy_check_mark:                    | :heavy_check_mark: |
| SaveFile     | :heavy_check_mark: | :white_check_mark: :heavy_check_mark: | :heavy_check_mark: |
|              |                    |                                       |                    |
| Filters      | :heavy_check_mark: | :white_check_mark: :heavy_check_mark: | :heavy_check_mark: |
| DefaultPath  |                    |                                       |                    |

[1] Freezes when used with winit (same way as `nfd`) [#1779](https://github.com/rust-windowing/winit/issues/1779)

- :white_check_mark: = Old `commdlg.h` API
- :heavy_check_mark: = Up to date API

# rfd-extras

AKA features that will be either in a separate `rfd-extras` crate, or behind a feature flag

| Feature       | Linux | Windows | MacOS |
| ------------- | ----- | ------- | ----- |
| MessageDialog |       |         |       |
| PromptDialog  |       |         |       |
| ColorPicker   |       |         |       |
