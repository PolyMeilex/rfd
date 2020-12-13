use crate::DialogOptions;
use std::path::Path;
use std::path::PathBuf;

use objc::{class, msg_send, sel, sel_impl};

use cocoa_foundation::base::id;
use cocoa_foundation::base::nil;
use cocoa_foundation::foundation::{NSArray, NSAutoreleasePool, NSString, NSURL};
use objc::runtime::{Object, YES};
pub use objc::runtime::{BOOL, NO};

pub fn pick_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let opt = params.into().unwrap_or_default();

    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::open_panel();

    if !opt.filters.is_empty() {
        panel.add_filters(&opt);
    }

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    panel.set_can_choose_directories(NO);
    panel.set_can_choose_files(YES);

    let res = if panel.run_modal() == 1 {
        Some(panel.get_result())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

pub fn save_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let opt = params.into().unwrap_or_default();

    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::save_panel();

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    let res = if panel.run_modal() == 1 {
        Some(panel.get_result())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

pub fn pick_folder<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let opt = params.into().unwrap_or_default();

    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::open_panel();

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    panel.set_can_choose_directories(YES);
    panel.set_can_choose_files(NO);

    let res = if panel.run_modal() == 1 {
        Some(panel.get_result())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

pub fn pick_files<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<Vec<PathBuf>> {
    let opt = params.into().unwrap_or_default();

    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::open_panel();

    if !opt.filters.is_empty() {
        panel.add_filters(&opt);
    }

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    panel.set_can_choose_directories(NO);
    panel.set_can_choose_files(YES);
    panel.set_allows_multiple_selection(YES);

    let res = if panel.run_modal() == 1 {
        Some(panel.get_results())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

//
// Internal
//

extern "C" {
    pub fn CGShieldingWindowLevel() -> i32;
}
fn make_nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s).autorelease() }
}

struct Panel {
    panel: *mut Object,
    _policy_manager: AppPolicyManager,
    key_window: *mut Object,
}
impl Panel {
    fn new(panel: *mut Object) -> Self {
        let _policy_manager = AppPolicyManager::new();
        let key_window = unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            msg_send![app, keyWindow]
        };

        let _: () = unsafe { msg_send![panel, setLevel: CGShieldingWindowLevel()] };
        Self {
            _policy_manager,
            panel,
            key_window,
        }
    }

    fn open_panel() -> Self {
        Self::new(unsafe { msg_send![class!(NSOpenPanel), openPanel] })
    }

    fn save_panel() -> Self {
        Self::new(unsafe { msg_send![class!(NSSavePanel), savePanel] })
    }

    fn run_modal(&self) -> i32 {
        unsafe { msg_send![self.panel, runModal] }
    }

    fn set_can_choose_directories(&self, v: BOOL) {
        let _: () = unsafe { msg_send![self.panel, setCanChooseDirectories: v] };
    }

    fn set_can_choose_files(&self, v: BOOL) {
        let _: () = unsafe { msg_send![self.panel, setCanChooseFiles: v] };
    }

    fn set_allows_multiple_selection(&self, v: BOOL) {
        let _: () = unsafe { msg_send![self.panel, setAllowsMultipleSelection: v] };
    }

    fn add_filters(&self, params: &DialogOptions) {
        let mut exts: Vec<&str> = Vec::new();

        for filter in params.filters.iter() {
            exts.append(&mut filter.extensions.to_vec());
        }

        unsafe {
            let f_raw: Vec<_> = exts.iter().map(|ext| make_nsstring(ext)).collect();

            let array = NSArray::arrayWithObjects(nil, f_raw.as_slice());
            let _: () = msg_send![self.panel, setAllowedFileTypes: array];
        }
    }

    fn set_path(&self, path: &Path) {
        if let Some(path) = path.to_str() {
            unsafe {
                let url = NSURL::alloc(nil)
                    .initFileURLWithPath_isDirectory_(make_nsstring(path), YES)
                    .autorelease();
                let () = msg_send![self.panel, setDirectoryURL: url];
            }
        }
    }

    fn get_result(&self) -> PathBuf {
        unsafe {
            let url: id = msg_send![self.panel, URL];
            let path: id = msg_send![url, path];
            let utf8: *const i32 = msg_send![path, UTF8String];
            let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

            let slice = std::slice::from_raw_parts(utf8 as *const _, len);
            let result = std::str::from_utf8_unchecked(slice);

            result.into()
        }
    }

    fn get_results(&self) -> Vec<PathBuf> {
        unsafe {
            let urls: id = msg_send![self.panel, URLs];

            let count = urls.count();

            let mut res = Vec::new();
            for id in 0..count {
                let url = urls.objectAtIndex(id);
                let path: id = msg_send![url, path];
                let utf8: *const i32 = msg_send![path, UTF8String];
                let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

                let slice = std::slice::from_raw_parts(utf8 as *const _, len);
                let result = std::str::from_utf8_unchecked(slice);
                res.push(result.into());
            }

            res
        }
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.key_window, makeKeyAndOrderFront: nil] };
    }
}

pub struct AppPolicyManager {
    initial_policy: i32,
}

impl AppPolicyManager {
    pub fn new() -> Self {
        #[repr(i32)]
        #[derive(Debug, PartialEq)]
        enum ApplicationActivationPolicy {
            //Regular = 0,
            Accessory = 1,
            Prohibited = 2,
            //Error = -1,
        }

        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let initial_policy: i32 = msg_send![app, activationPolicy];

            if initial_policy == ApplicationActivationPolicy::Prohibited as i32 {
                let new_pol = ApplicationActivationPolicy::Accessory as i32;
                let _: () = msg_send![app, setActivationPolicy: new_pol];
            }

            Self { initial_policy }
        }
    }
}
impl Drop for AppPolicyManager {
    fn drop(&mut self) {
        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            // Restore initial pol
            let _: () = msg_send![app, setActivationPolicy: self.initial_policy];
        }
    }
}
