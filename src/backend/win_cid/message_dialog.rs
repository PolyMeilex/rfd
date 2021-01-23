use super::thread_future::ThreadFuture;
use crate::dialog::{MessageButtons, MessageDialog, MessageLevel};

use winapi::um::winuser::{
    MessageBoxW, IDOK, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_OKCANCEL,
    MB_YESNO,
};

use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt, ptr};

pub struct WinMessageDialog {
    text: Vec<u16>,
    caption: Vec<u16>,
    flags: u32,
}

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
            MessageButtons::OkCancle => MB_OKCANCEL,
            MessageButtons::YesNo => MB_YESNO,
        };

        Self {
            text,
            caption,
            flags: level | buttons,
        }
    }

    pub fn run(self) -> bool {
        let ret = unsafe {
            MessageBoxW(
                ptr::null_mut(),
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
