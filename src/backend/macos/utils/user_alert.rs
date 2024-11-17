use ::objc2::rc::autoreleasepool;
use ::objc2_foundation::MainThreadMarker;
use ::deferred_future::ThreadDeferredFuture;
use ::std::{ mem::MaybeUninit, ptr, sync::PoisonError };
use ::core_foundation::{ base::TCFType, string::CFString };
use ::futures::{ future, executor::ThreadPool, task::SpawnExt };
use ::core_foundation_sys::{ base::CFOptionFlags, date::CFTimeInterval, url::CFURLRef, user_notification::{ CFUserNotificationDisplayAlert, kCFUserNotificationStopAlertLevel, kCFUserNotificationCautionAlertLevel, kCFUserNotificationNoteAlertLevel,  kCFUserNotificationDefaultResponse, kCFUserNotificationAlternateResponse, kCFUserNotificationOtherResponse } };
use crate::{
    message_dialog::{ MessageButtons, MessageDialog, MessageDialogResult, MessageLevel },
    backend::{ DialogFutureType, macos::utils::{ FocusManager, PolicyManager, run_on_main } }
};
struct UserAlert {
    timeout: CFTimeInterval,
    flags: CFOptionFlags,
    icon_url: CFURLRef,
    sound_url: CFURLRef,
    localization_url: CFURLRef,
    alert_header: String,
    alert_message: String,
    default_button_title: Option<String>,
    alternate_button_title: Option<String>,
    other_button_title: Option<String>,
    buttons: MessageButtons,
    _focus_manager: Option<FocusManager>,
    _policy_manager: Option<PolicyManager>,
}
impl UserAlert {
    pub fn new(opt: MessageDialog, mtm: Option<MainThreadMarker>) -> Self {
        let mut buttons: [Option<String>; 3] = match &opt.buttons {
            MessageButtons::Ok => [None, None, None],
            MessageButtons::OkCancel => [None, Some("Cancel".to_string()), None],
            MessageButtons::YesNo => [Some("Yes".to_string()), Some("No".to_string()), None],
            MessageButtons::YesNoCancel => [Some("Yes".to_string()), Some("No".to_string()), Some("Cancel".to_string())],
            MessageButtons::OkCustom(ok_text) => [Some(ok_text.to_string()), None, None],
            MessageButtons::OkCancelCustom(ok_text, cancel_text) => [Some(ok_text.to_string()), Some(cancel_text.to_string()), None],
            MessageButtons::YesNoCancelCustom(yes_text, no_text, cancel_text) => [Some(yes_text.to_string()), Some(no_text.to_string()), Some(cancel_text.to_string())]
        };
        UserAlert {
            timeout: 0_f64,
            icon_url: ptr::null(),
            sound_url: ptr::null(),
            localization_url: ptr::null(),
            flags: match opt.level {
                MessageLevel::Info => kCFUserNotificationNoteAlertLevel,
                MessageLevel::Warning => kCFUserNotificationCautionAlertLevel,
                MessageLevel::Error => kCFUserNotificationStopAlertLevel
            },
            alert_header: opt.title,
            alert_message: opt.description,
            default_button_title: buttons[0].take(),
            alternate_button_title: buttons[1].take(),
            other_button_title: buttons[2].take(),
            buttons: opt.buttons,
            _policy_manager: mtm.map(|mtm| PolicyManager::new(mtm)),
            _focus_manager: mtm.map(|mtm| FocusManager::new(mtm))
        }
    }
    pub fn run(self) -> MessageDialogResult {
        let alert_header =  CFString::new(&self.alert_header[..]);
        let alert_message = CFString::new(&self.alert_message[..]);
        let default_button_title = self.default_button_title.map(|string| CFString::new(&string[..]));
        let alternate_button_title = self.alternate_button_title.map(|value| CFString::new(&value[..]));
        let other_button_title = self.other_button_title.map(|value| CFString::new(&value[..]));
        let mut response_flags = MaybeUninit::<CFOptionFlags>::uninit();
        let is_canel = unsafe { CFUserNotificationDisplayAlert(
            self.timeout,
            self.flags,
            self.icon_url,
            self.sound_url,
            self.localization_url,
            alert_header.as_concrete_TypeRef(),
            alert_message.as_concrete_TypeRef(),
            default_button_title.map_or(ptr::null(), |value| value.as_concrete_TypeRef()),
            alternate_button_title.map_or(ptr::null(), |value| value.as_concrete_TypeRef()),
            other_button_title.map_or(ptr::null(), |value| value.as_concrete_TypeRef()),
            response_flags.as_mut_ptr()
        ) };
        if is_canel != 0 {
            return MessageDialogResult::Cancel;
        }
        let response = unsafe { response_flags.assume_init() };
        match self.buttons {
            MessageButtons::Ok if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Ok,
            MessageButtons::OkCancel if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Ok,
            MessageButtons::OkCancel if response == kCFUserNotificationAlternateResponse => MessageDialogResult::Cancel,
            MessageButtons::YesNo if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Yes,
            MessageButtons::YesNo if response == kCFUserNotificationAlternateResponse => MessageDialogResult::No,
            MessageButtons::YesNoCancel if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Yes,
            MessageButtons::YesNoCancel if response == kCFUserNotificationAlternateResponse => MessageDialogResult::No,
            MessageButtons::YesNoCancel if response == kCFUserNotificationOtherResponse => MessageDialogResult::Cancel,
            MessageButtons::OkCustom(custom) if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Custom(custom.to_owned()),
            MessageButtons::OkCancelCustom(custom, _) if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Custom(custom.to_owned()),
            MessageButtons::OkCancelCustom(_, custom) if response == kCFUserNotificationAlternateResponse => MessageDialogResult::Custom(custom.to_owned()),
            MessageButtons::YesNoCancelCustom(custom, _, _) if response == kCFUserNotificationDefaultResponse => MessageDialogResult::Custom(custom.to_owned()),
            MessageButtons::YesNoCancelCustom(_, custom, _) if response == kCFUserNotificationAlternateResponse => MessageDialogResult::Custom(custom.to_owned()),
            MessageButtons::YesNoCancelCustom(_, _, custom) if response == kCFUserNotificationOtherResponse => MessageDialogResult::Custom(custom.to_owned()),
            _ => MessageDialogResult::Cancel,
        }
    }
}
pub fn sync_pop_dialog(opt: MessageDialog, mtm: MainThreadMarker) -> MessageDialogResult {
    UserAlert::new(opt, Some(mtm)).run()
}
pub fn async_pop_dialog(opt: MessageDialog) -> DialogFutureType<MessageDialogResult> {
    let deferred_future = ThreadDeferredFuture::default();
    let defer = deferred_future.defer();
    let opt2 = opt.clone();
    let result: Result<(), String> = ThreadPool::new().map_err(|err| err.to_string()).and_then(|thread_pool| thread_pool.spawn(async move {
        let mut defer = defer.lock().unwrap_or_else(PoisonError::into_inner);
        let message_dialog_result = UserAlert::new(opt2.clone(), None).run();
        defer.complete(message_dialog_result);
    }).map_err(|err| err.to_string()));
    match result {
        Ok(_) => Box::pin(deferred_future),
        Err(err) => {
            eprintln!("\n Hi! It looks like you are running async dialog in unsupported environment, I will fallback to sync dialog for you. Reason is as below:\n {}", err);
            Box::pin(future::ready(autoreleasepool(move |_| run_on_main(move |mtm| UserAlert::new(opt, Some(mtm)).run()))))
        }
    }
}