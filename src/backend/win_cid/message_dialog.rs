use super::thread_future::ThreadFuture;
use crate::message_dialog::{MessageButtons, MessageDialog, MessageLevel};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{IDOK, IDYES},
    },
};

#[cfg(not(feature = "common-controls-v6"))]
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_OKCANCEL, MB_YESNO,
    MESSAGEBOX_STYLE,
};

use raw_window_handle::RawWindowHandle;

use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

pub struct WinMessageDialog {
    parent: Option<HWND>,
    text: Vec<u16>,
    caption: Vec<u16>,
    #[cfg(not(feature = "common-controls-v6"))]
    flags: MESSAGEBOX_STYLE,
    #[cfg(feature = "common-controls-v6")]
    opt: MessageDialog,
}

// Oh god, I don't like sending RawWindowHandle between threads but here we go anyways...
// fingers crossed
unsafe impl Send for WinMessageDialog {}

fn str_to_vec_u16(str: &str) -> Vec<u16> {
    OsStr::new(str).encode_wide().chain(once(0)).collect()
}

impl WinMessageDialog {
    pub fn new(opt: MessageDialog) -> Self {
        let text: Vec<u16> = str_to_vec_u16(&opt.description);
        let caption: Vec<u16> = str_to_vec_u16(&opt.title);

        #[cfg(not(feature = "common-controls-v6"))]
        let level = match opt.level {
            MessageLevel::Info => MB_ICONINFORMATION,
            MessageLevel::Warning => MB_ICONWARNING,
            MessageLevel::Error => MB_ICONERROR,
        };

        #[cfg(not(feature = "common-controls-v6"))]
        let buttons = match opt.buttons {
            MessageButtons::Ok | MessageButtons::OkCustom(_) => MB_OK,
            MessageButtons::OkCancel | MessageButtons::OkCancelCustom(_, _) => MB_OKCANCEL,
            MessageButtons::YesNo => MB_YESNO,
        };

        let parent = match opt.parent {
            Some(RawWindowHandle::Win32(handle)) => Some(HWND(handle.hwnd as _)),
            None => None,
            _ => unreachable!("unsupported window handle, expected: Windows"),
        };

        Self {
            parent,
            text,
            caption,
            #[cfg(not(feature = "common-controls-v6"))]
            flags: level | buttons,
            #[cfg(feature = "common-controls-v6")]
            opt,
        }
    }

    #[cfg(feature = "common-controls-v6")]
    pub fn run(mut self) -> bool {
        use windows::Win32::{
            Foundation::BOOL,
            UI::Controls::{
                TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOG_BUTTON,
                TASKDIALOG_COMMON_BUTTON_FLAGS, TDCBF_CANCEL_BUTTON, TDCBF_NO_BUTTON,
                TDCBF_OK_BUTTON, TDCBF_YES_BUTTON, TDF_ALLOW_DIALOG_CANCELLATION,
            },
        };

        let mut pf_verification_flag_checked = BOOL(0);
        let mut pn_button = 0;
        let mut pn_radio_button = 0;

        let id_custom_ok = 1000;
        let id_custom_cancel = 1001;

        let mut task_dialog_config = TASKDIALOGCONFIG {
            cbSize: core::mem::size_of::<TASKDIALOGCONFIG>() as u32,
            hwndParent: self.parent.unwrap_or_default(),
            dwFlags: TDF_ALLOW_DIALOG_CANCELLATION,
            cButtons: 0,
            pszWindowTitle: PCWSTR(self.caption.as_mut_ptr()),
            pszContent: PCWSTR(self.text.as_mut_ptr()),
            ..Default::default()
        };

        let main_icon_ptr = match self.opt.level {
            // `TD_WARNING_ICON` / `TD_ERROR_ICON` / `TD_INFORMATION_ICON` are missing in windows-rs
            // https://github.com/microsoft/win32metadata/issues/968
            // Workaround via hard code:
            // TD_WARNING_ICON
            MessageLevel::Warning => -1 as i16 as u16,
            // TD_ERROR_ICON
            MessageLevel::Error => -2 as i16 as u16,
            // TD_INFORMATION_ICON
            MessageLevel::Info => -3 as i16 as u16,
        };

        task_dialog_config.Anonymous1.pszMainIcon = PCWSTR(main_icon_ptr as *const u16);

        let (system_buttons, custom_buttons) = match self.opt.buttons {
            MessageButtons::Ok => (TDCBF_OK_BUTTON, vec![]),
            MessageButtons::OkCancel => (
                TASKDIALOG_COMMON_BUTTON_FLAGS(TDCBF_OK_BUTTON.0 | TDCBF_CANCEL_BUTTON.0),
                vec![],
            ),
            MessageButtons::YesNo => (
                TASKDIALOG_COMMON_BUTTON_FLAGS(TDCBF_YES_BUTTON.0 | TDCBF_NO_BUTTON.0),
                vec![],
            ),
            MessageButtons::OkCustom(ok_text) => (
                Default::default(),
                vec![(id_custom_ok, str_to_vec_u16(&ok_text))],
            ),
            MessageButtons::OkCancelCustom(ok_text, cancel_text) => (
                Default::default(),
                vec![
                    (id_custom_ok, str_to_vec_u16(&ok_text)),
                    (id_custom_cancel, str_to_vec_u16(&cancel_text)),
                ],
            ),
        };

        let p_buttons = custom_buttons
            .iter()
            .map(|(id, text)| TASKDIALOG_BUTTON {
                nButtonID: *id,
                pszButtonText: PCWSTR(text.as_ptr()),
            })
            .collect::<Vec<_>>();
        task_dialog_config.dwCommonButtons = system_buttons;
        task_dialog_config.pButtons = p_buttons.as_ptr();
        task_dialog_config.cButtons = custom_buttons.len() as u32;

        let ret = unsafe {
            TaskDialogIndirect(
                &task_dialog_config,
                &mut pn_button,
                &mut pn_radio_button,
                &mut pf_verification_flag_checked,
            )
        };

        ret.is_ok() && (pn_button == id_custom_ok || pn_button == IDYES.0 || pn_button == IDOK.0)
    }

    #[cfg(not(feature = "common-controls-v6"))]
    pub fn run(mut self) -> bool {
        let ret = unsafe {
            MessageBoxW(
                self.parent,
                PCWSTR(self.text.as_mut_ptr()),
                PCWSTR(self.caption.as_mut_ptr()),
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
