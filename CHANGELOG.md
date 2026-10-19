# Change Log

## Unreleased

- Fix some GTK3 memory leaks
- Add GTK4 backend behind the `gtk4` feature
- MSRV is now 1.92
- Fix macOS file dialogs ignoring `set_directory` when `set_file_name` is also used.
- Add `FileDialog::set_show_hidden_files` and `AsyncFileDialog::set_show_hidden_files` to control hidden file visibility. Supported on macOS, Windows, and Linux (GTK3).
- Honor `FileDialog::set_directory` and `AsyncFileDialog::set_directory` when using the zenity fallback backend on Linux.
- Fix the zenity fallback returning `None` for a file name that isn't valid UTF-8. zenity's output is now kept as bytes.
- Return `None` instead of `Some(vec![])` when `pick_files` or `pick_folders` is cancelled in the zenity fallback, as the XDG portal backend does since #323.
- Fix filter names that contain `|` in the zenity fallback, which zenity cut off at the `|`, reading the rest as file patterns. `|` in a filter name is now shown as `/`.
- Open the directory set with `set_directory` in the zenity fallback's open and folder dialogs, which could start on Recent with GTK's default file chooser settings. The directory is now passed with `--filename` rather than as zenity's working directory, and a directory that doesn't exist no longer stops the dialog from opening.
- Fix `liblary` typo in docs
- Change wasm title to sanitize text contents via `title_el.set_text_content()` rather than `title_el.set_inner_html()`.
- Set wasm dialog styles via `set_text_content()` rather than `set_inner_html()`, so rfd no longer uses `innerHTML`.
- Fix GTK3 message dialogs using the description as a printf format string, which garbled text containing `%` and could crash. The description is now shown as given, so `%%` is no longer shown as `%`.
- Support files without an extension in GTK3 and zenity filters, as the XDG portal backend does since #215. A match-all filter (`"*"`) or an empty one (`""`) now uses the glob `"*"`, not `"*.*"` or `"*."`, so a filter such as "All files" matches a name with no dot, like `README`.
- Zenity now returns correct paths when a name contains `|`. Also file names that start or end with a space are kept intact.

## 0.17.2

- Lower MSRV back to 1.88 by @PolyMeilex in https://github.com/PolyMeilex/rfd/pull/303

## 0.17.1

- Fix aarch64 compile error and add CI by @oscargus in https://github.com/PolyMeilex/rfd/pull/294
- docs: Fixup docs post libdbus port by @PolyMeilex in https://github.com/PolyMeilex/rfd/pull/298
- xdg_porta: Url decode the path by @PolyMeilex in https://github.com/PolyMeilex/rfd/pull/297

## 0.17.0

- `tokio` and `async-std` features removed

## 0.16.0

- Fix regressions on Wayland due to `ashpd` upgrade (#255).
- The `pick_file()` method of file dialog targeted WASM now can return `None` correctly when cancelled (#258)
- Update `windows-sys` to 0.60.
- Make `ashpd` Wayland APIs optional. These are now gated behind the `wayland` feature, which is enabled by default.

### Changed items in the public API
```diff
-pub fn AsyncFileDialog::set_parent<W: HasWindowHandle + HasDisplayHandle>(self, parent: &W) -> Self
+pub fn AsyncFileDialog::set_parent<W: HasWindowHandle + HasDisplayHandle + ?Sized>(self, parent: &W) -> Self
-pub fn AsyncMessageDialog::set_parent<W: HasWindowHandle + HasDisplayHandle>(self, parent: &W) -> Self
+pub fn MessageDialog::set_parent<W: HasWindowHandle + HasDisplayHandle + ?Sized>(self, parent: &W) -> Self
-pub fn FileDialog::set_parent<W: HasWindowHandle + HasDisplayHandle>(self, parent: &W) -> Self
+pub fn rfd::FileDialog::set_parent<W: HasWindowHandle + HasDisplayHandle + ?Sized>(self, parent: &W) -> Self
-pub fn MessageDialog::set_parent<W: HasWindowHandle + HasDisplayHandle>(self, parent: &W) -> Self
+pub fn MessageDialog::set_parent<W: HasWindowHandle + HasDisplayHandle + ?Sized>(self, parent: &W) -> Self
```

## 0.15.3

- Update `objc2` to v0.6.
- Update `ashpd` to 0.11.

## 0.15.1

- Update `ashpd` to 0.10.
- Fix issue where with no filter added no files are selectable on Windows (#211).

## 0.15.0

- Move from `objc` crates to `objc2` crates.
- Fix `AsyncFileDialog` blocking the executor on Windows (#191)
- Add `TDF_SIZE_TO_CONTENT` to `TaskDialogIndirect` config so that it can display longer text without truncating/wrapping (80 characters instead of 55) (#202)
- Fix `xdg-portal` backend not accepting special characters in message dialogs
- Make `set_parent` require `HasWindowHandle + HasDisplayHandle`
- Add support for `set_parent` in XDG Portals
- Update `ashpd` to 0.9.
- Add support for files without an extension in XDG Portal filters
- Derive `Clone` for `FileHandle`

## 0.14.0

- i18n for GTK and XDG Portal
- Use XDG Portal as default
- Use zenity as a fallback for XDG Portal
- Update `raw-window-handle` to 0.6.
- Update `winit` in example to 0.29.
- Update `ashpd` to 0.8.
- Update wasm CSS to respect the color scheme (including dark mode)
- Fix macOS sync backend incorrectly setting the parent window
- Add `FileDialog/AsyncFileDialog::set_can_create_directories`, supported on macOS only.

## 0.13.0

- **[Breaking]** Users of the `xdg-portal` feature must now also select the `tokio`
  or `async-std` feature
- [macOS] Use NSOpenPanel.message instead of title #166

## 0.12.1

- Fix `FileHandle::inner` (under feature `file-handle-inner`) on wasm

## 0.12.0

- Add title support for WASM (#132)
- Add Create folder button to `pick_folder` on macOS (#127)
- Add support for Yes/No/Cancel buttons (#123)
- Change a string method signatures #117
- WASM `save_file` (#134)
- Update `gtk-sys` to `0.18` (#143)
- Update `ashpd` to `0.6` (#133)
- Replace windows with `windows-sys` (#118)
- Make zenity related deps optional (#141)

## 0.11.3

- Zenity message dialogs for xdg portal backend

## 0.10.1

- Update `gtk-sys` to `0.16` and `windows-rs` to `0.44`

## 0.10.0

- fix(FileDialog::set_directory): fallback to default if path is empty

## 0.9.0

- feat: customize button text, Close #74
- feat: Add support for selecting multiple folders, fixes #73

## 0.8.4

- XDG: decode URI before converting to PathBuf #70

## 0.8.3

- Windows-rs update 0.37

## 0.8.2

- Windows-rs update 0.35

## 0.8.1

- Macos parent for sync FileDialog (#58)
- Windows-rs update 0.33

## 0.8.0

- `parent` feature was removed, it is always on now
- New feature `xdg-portal`
- Now you have to choose one of the features `gtk3` or `xdg-portal`, gtk is on by default
- `window` crate got updated to 0.32

## 0.7.0

- Safe Rust XDG Desktop Portal support

## 0.6.3

- Update `windows` crate to 0.30.

## 0.6.2

- Strip Win32 namespaces from directory paths

## 0.6.0

- FreeBSD support
- Port to windows-rs
- Update RawWindowHandle to 0.4

## 0.4.4

- Fix `set_directory` on some windows setups (#22)
- Implement `set_file_name` on MacOS (#21)

## 0.4.3

- `set_parent` support for `MessageDialog` on windows

## 0.4.2

- GTK save dialog now sets current_name correctly (#18)

## 0.4.1

- Update gtk

## 0.4.0

- **[Breaking]** Fix misspeled `OkCancel` in `MessageButtons` (#12)
- `set_parent` support for Windows (#14)
