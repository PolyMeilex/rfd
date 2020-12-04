# rfd

Very WIP and untested native file dialogs for Windows, Linux (GTK), MacOS.

# Examples

All examples are located in `examples` directory.

- Run `cargo run --example simple` for the simple example.
- Run `cargo run --example filter` for an example utilizing a filter.

# State

| Feature      | Linux              | Windows            | MacOS                  |
| ------------ | ------------------ | ------------------ | ---------------------- |
| SingleFile   | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: [1] |
| MultipleFile | :heavy_check_mark: | :construction:     | :construction:         |
| PickFolder   |                    |                    |                        |
| SaveFile     |                    |                    |                        |

[1] Freezes when used with winit (same way as `nfd`) [#1779](https://github.com/rust-windowing/winit/issues/1779)
