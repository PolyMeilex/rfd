use crate::backend::AsyncMessageDialogImpl;
use crate::backend::MessageDialogImpl;

use std::future::Future;

#[cfg(feature = "parent")]
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// Synchronous Message Dialog. Supported platforms:
///  * Windows
///  * macOS
///  * Linux (GTK only)
///  * WASM
#[derive(Default, Debug, Clone)]
pub struct MessageDialog {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) level: MessageLevel,
    pub(crate) buttons: MessageButtons,
    #[cfg(feature = "parent")]
    pub(crate) parent: Option<RawWindowHandle>,
}

// Oh god, I don't like sending RawWindowHandle between threads but here we go anyways...
// fingers crossed
unsafe impl Send for MessageDialog {}

impl MessageDialog {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set level of a dialog
    ///
    /// Depending on the system it can result in level specific icon to show up,
    /// the will inform user it message is a error, warning or just information.
    pub fn set_level(mut self, level: MessageLevel) -> Self {
        self.level = level;
        self
    }

    /// Set title of a dialog
    pub fn set_title(mut self, text: &str) -> Self {
        self.title = text.into();
        self
    }

    /// Set description of a dialog
    ///
    /// Description is a content of a dialog
    pub fn set_description(mut self, text: &str) -> Self {
        self.description = text.into();
        self
    }

    /// Set the set of button that will be displayed on the dialog
    ///
    /// - `Ok` dialog is a single `Ok` button
    /// - `OkCancel` dialog, will display 2 buttons ok and cancel.
    /// - `YesNo` dialog, will display 2 buttons yes and no.
    pub fn set_buttons(mut self, btn: MessageButtons) -> Self {
        self.buttons = btn;
        self
    }

    #[cfg(feature = "parent")]
    /// Set parent windows explicitly (optional)
    /// Suported in: `macos` and `windows`
    pub fn set_parent<W: HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.parent = Some(parent.raw_window_handle());
        self
    }

    /// Shows a message dialog:
    ///
    /// - In `Ok` dialog, it will return `true` when `OK` was pressed
    /// - In `OkCancel` dialog, it will return `true` when `OK` was pressed
    /// - In `YesNo` dialog, it will return `true` when `Yes` was pressed
    pub fn show(self) -> bool {
        MessageDialogImpl::show(self)
    }
}

/// Asynchronous Message Dialog. Supported platforms:
///  * Windows
///  * macOS
///  * Linux (GTK only)
///  * WASM
#[derive(Default, Debug, Clone)]
pub struct AsyncMessageDialog(MessageDialog);

impl AsyncMessageDialog {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set level of a dialog
    ///
    /// Depending on the system it can result in level specific icon to show up,
    /// the will inform user it message is a error, warning or just information.
    pub fn set_level(mut self, level: MessageLevel) -> Self {
        self.0 = self.0.set_level(level);
        self
    }

    /// Set title of a dialog
    pub fn set_title(mut self, text: &str) -> Self {
        self.0 = self.0.set_title(text);
        self
    }

    /// Set description of a dialog
    ///
    /// Description is a content of a dialog
    pub fn set_description(mut self, text: &str) -> Self {
        self.0 = self.0.set_description(text);
        self
    }

    /// Set the set of button that will be displayed on the dialog
    ///
    /// - `Ok` dialog is a single `Ok` button
    /// - `OkCancel` dialog, will display 2 buttons ok and cancel.
    /// - `YesNo` dialog, will display 2 buttons yes and no.
    pub fn set_buttons(mut self, btn: MessageButtons) -> Self {
        self.0 = self.0.set_buttons(btn);
        self
    }

    #[cfg(feature = "parent")]
    /// Set parent windows explicitly (optional)
    /// Suported in: `macos` and `windows`
    pub fn set_parent<W: HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.0 = self.0.set_parent(parent);
        self
    }

    /// Shows a message dialog:
    /// - In `Ok` dialog, it will return `true` when `OK` was pressed
    /// - In `OkCancel` dialog, it will return `true` when `OK` was pressed
    /// - In `YesNo` dialog, it will return `true` when `Yes` was pressed
    pub fn show(self) -> impl Future<Output = bool> {
        AsyncMessageDialogImpl::show_async(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

impl Default for MessageLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageButtons {
    Ok,
    OkCancel,
    YesNo,
}

impl Default for MessageButtons {
    fn default() -> Self {
        Self::Ok
    }
}
