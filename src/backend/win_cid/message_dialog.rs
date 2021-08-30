use super::thread_future::ThreadFuture;
use crate::dialog::{MessageButtons, MessageDialog, MessageLevel};

use winapi::um::winuser::{
    MessageBoxW, IDOK, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_OKCANCEL,
    MB_YESNO,
};

#[cfg(feature = "parent")]
use raw_window_handle::RawWindowHandle;

use std::{
    ffi::{c_void, OsStr},
    iter::once,
    os::windows::ffi::OsStrExt,
    ptr,
};

pub struct WinMessageDialog {
    parent: Option<*mut c_void>,
    text: Vec<u16>,
    caption: Vec<u16>,
    flags: u32,
}

// Oh god, I don't like sending RawWindowHandle between threads but here we go anyways...
// fingers crossed
unsafe impl Send for WinMessageDialog {}

impl WinMessageDialog {
    pub fn new(opt: MessageDialog) -> Self {
        let input = format!("{}\n{}", opt.title, opt.description);
        let text: Vec<u16> = OsStr::new(&input).encode_wide().chain(once(0)).collect();
        let caption: Vec<u16> = OsStr::new(&opt.title)
            .encode_wide()
            .chain(once(0))
            .collect();

        let level = match opt.level {
            MessageLevel::Info => MB_ICONINFORMATION,
            MessageLevel::Warning => MB_ICONWARNING,
            MessageLevel::Error => MB_ICONERROR,
        };

        let buttons = match opt.buttons {
            MessageButtons::Ok => MB_OK,
            MessageButtons::OkCancel => MB_OKCANCEL,
            MessageButtons::YesNo => MB_YESNO,
        };

        #[cfg(feature = "parent")]
        let parent = match opt.parent {
            Some(RawWindowHandle::Windows(handle)) => Some(handle.hwnd),
            None => None,
            _ => unreachable!("unsupported window handle, expected: Windows"),
        };
        #[cfg(not(feature = "parent"))]
        let parent = None;

        Self {
            parent,
            text,
            caption,
            flags: level | buttons,
        }
    }

    pub fn run(self) -> bool {
        let ret = unsafe {
            MessageBoxW(
                self.parent.unwrap_or_else(|| ptr::null_mut()) as _,
                self.text.as_ptr(),
                self.caption.as_ptr(),
                self.flags,
            )
        };

        ret == IDOK || ret == IDYES
    }

    pub fn run_async(self) -> ThreadFuture<bool> {
        ThreadFuture::new(move |data| *data = Some(self.run()))
    }
}

use crate::backend::MessageDialogImpl;

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> bool {
        let dialog = WinMessageDialog::new(self);
        dialog.run()
    }
}

use crate::backend::AsyncMessageDialogImpl;
use crate::backend::DialogFutureType;

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<bool> {
        let dialog = WinMessageDialog::new(self);
        Box::pin(dialog.run_async())
    }
}
