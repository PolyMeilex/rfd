# Change Log

## Unreleased

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
