use super::thread_future::ThreadFuture;
use super::utils::str_to_vec_u16;
use crate::message_dialog::{MessageButtons, MessageDialog, MessageLevel};

use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{IDOK, IDYES},
};

#[cfg(not(feature = "common-controls-v6"))]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_OKCANCEL, MB_YESNO,
    MESSAGEBOX_STYLE,
};

use raw_window_handle::RawWindowHandle;

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
            Some(RawWindowHandle::Win32(handle)) => Some(handle.hwnd as _),
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
    pub fn run(self) -> bool {
        use windows_sys::Win32::UI::Controls::{
            TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TASKDIALOGCONFIG_1,
            TASKDIALOG_BUTTON, TDCBF_CANCEL_BUTTON, TDCBF_NO_BUTTON, TDCBF_OK_BUTTON,
            TDCBF_YES_BUTTON, TDF_ALLOW_DIALOG_CANCELLATION, TD_ERROR_ICON, TD_INFORMATION_ICON,
            TD_WARNING_ICON,
        };

        let mut pf_verification_flag_checked = 0;
        let mut pn_button = 0;
        let mut pn_radio_button = 0;

        let id_custom_ok = 1000;
        let id_custom_cancel = 1001;

        let main_icon_ptr = match self.opt.level {
            MessageLevel::Warning => TD_WARNING_ICON,
            MessageLevel::Error => TD_ERROR_ICON,
            MessageLevel::Info => TD_INFORMATION_ICON,
        };

        let (system_buttons, custom_buttons) = match self.opt.buttons {
            MessageButtons::Ok => (TDCBF_OK_BUTTON, vec![]),
            MessageButtons::OkCancel => (TDCBF_OK_BUTTON | TDCBF_CANCEL_BUTTON, vec![]),
            MessageButtons::YesNo => (TDCBF_YES_BUTTON | TDCBF_NO_BUTTON, vec![]),
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
                pszButtonText: text.as_ptr(),
            })
            .collect::<Vec<_>>();

        let task_dialog_config = TASKDIALOGCONFIG {
            cbSize: core::mem::size_of::<TASKDIALOGCONFIG>() as u32,
            hwndParent: self.parent.unwrap_or_default(),
            dwFlags: TDF_ALLOW_DIALOG_CANCELLATION,
            pszWindowTitle: self.caption.as_ptr(),
            pszContent: self.text.as_ptr(),
            Anonymous1: TASKDIALOGCONFIG_0 {
                pszMainIcon: main_icon_ptr,
            },
            Anonymous2: TASKDIALOGCONFIG_1 {
                pszFooterIcon: std::ptr::null(),
            },
            dwCommonButtons: system_buttons,
            pButtons: p_buttons.as_ptr(),
            cButtons: custom_buttons.len() as u32,
            pRadioButtons: std::ptr::null(),
            cRadioButtons: 0,
            cxWidth: 0,
            hInstance: 0,
            pfCallback: None,
            lpCallbackData: 0,
            nDefaultButton: 0,
            nDefaultRadioButton: 0,
            pszCollapsedControlText: std::ptr::null(),
            pszExpandedControlText: std::ptr::null(),
            pszExpandedInformation: std::ptr::null(),
            pszMainInstruction: std::ptr::null(),
            pszVerificationText: std::ptr::null(),
            pszFooter: std::ptr::null(),
        };

        let ret = unsafe {
            TaskDialogIndirect(
                &task_dialog_config,
                &mut pn_button,
                &mut pn_radio_button,
                &mut pf_verification_flag_checked,
            )
        };

        ret == 0 && (pn_button == id_custom_ok || pn_button == IDYES || pn_button == IDOK)
    }

    #[cfg(not(feature = "common-controls-v6"))]
    pub fn run(self) -> bool {
        let ret = unsafe {
            MessageBoxW(
                self.parent.unwrap_or_default(),
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
