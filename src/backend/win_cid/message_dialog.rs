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
        let text: Vec<u16> = OsStr::new(&opt.text).encode_wide().chain(once(0)).collect();
        let caption: Vec<u16> = OsStr::new("").encode_wide().chain(once(0)).collect();

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
}

use crate::backend::MessageDialogImpl;

impl MessageDialogImpl for MessageDialog {
    fn show(self) {
        let dialog = WinMessageDialog::new(self);
        let res = dialog.run();

        println!("{:?}", res);
    }
}
