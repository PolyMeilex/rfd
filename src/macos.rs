use crate::DialogOptions;
use std::path::PathBuf;

use objc::{msg_send, sel, sel_impl};

use cocoa_foundation::base::nil;
use cocoa_foundation::foundation::NSAutoreleasePool;
pub use objc::runtime::{BOOL, NO, YES};

mod utils {
    use crate::DialogParams;

    use std::path::PathBuf;

    use cocoa_foundation::base::{id, nil};
    use cocoa_foundation::foundation::{NSArray, NSAutoreleasePool, NSString, NSURL};
    use objc::runtime::{Object, YES};
    use objc::{class, msg_send, sel, sel_impl};

    extern "C" {
        pub fn CGShieldingWindowLevel() -> i32;
    }

    pub unsafe fn app() -> *mut Object {
        msg_send![class!(NSApplication), sharedApplication]
    }

    pub unsafe fn key_window() -> *mut Object {
        let app = app();
        msg_send![app, keyWindow]
    }

    pub fn open_panel() -> *mut Object {
        unsafe { msg_send![class!(NSOpenPanel), openPanel] }
    }

    pub fn save_panel() -> *mut Object {
        unsafe { msg_send![class!(NSSavePanel), savePanel] }
    }

    pub fn make_nsstring(s: &str) -> id {
        unsafe { NSString::alloc(nil).init_str(s).autorelease() }
    }

    pub unsafe fn add_filters(panel: id, params: &mut DialogParams) {
        let mut exts = Vec::new();
        
        for filter in params.filters.iter_mut(){
            exts.append(&mut filter.extensions);
        }

        let f_raw: Vec<_> = exts.iter().map(|ext| make_nsstring(ext)).collect();

        let array = NSArray::arrayWithObjects(nil, f_raw.as_slice());
        let _: () = msg_send![panel, setAllowedFileTypes: array];
    }

    pub unsafe fn set_path(panel: id, path: &PathBuf) {
        if let Some(path) = path.to_str() {
            let url = NSURL::alloc(nil)
                .initFileURLWithPath_isDirectory_(make_nsstring(path), YES)
                .autorelease();
            let () = msg_send![panel, setDirectoryURL: url];
        }
    }

    pub unsafe fn get_result(panel: id) -> PathBuf {
        let url: id = msg_send![panel, URL];
        let path: id = msg_send![url, path];
        let utf8: *const i32 = msg_send![path, UTF8String];
        let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

        let slice = std::slice::from_raw_parts(utf8 as *const _, len);
        let result = std::str::from_utf8_unchecked(slice);

        result.into()
    }

    pub unsafe fn get_results(panel: id) -> Vec<PathBuf> {
        let urls: id = msg_send![panel, URLs];

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

    #[repr(i32)]
    #[derive(Debug, PartialEq)]
    enum ApplicationActivationPolicy {
        //Regular = 0,
        Accessory = 1,
        Prohibited = 2,
        //Error = -1,
    }

    pub struct AppPolicyManager {
        initial_policy: i32,
    }

    impl AppPolicyManager {
        pub fn new() -> Self {
            unsafe {
                let app = app();
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
                let app = app();
                // Restore initial pol
                let _: () = msg_send![app, setActivationPolicy: self.initial_policy];
            }
        }
    }
}

use utils::*;

pub fn pick_file<'a>(mut params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let key_window = key_window();

        let _policy_manager = AppPolicyManager::new();

        let res = {
            let panel = open_panel();

            let _: () = msg_send![panel, setLevel: CGShieldingWindowLevel()];

            let _: () = msg_send![panel, setCanChooseDirectories: NO];
            let _: () = msg_send![panel, setCanChooseFiles: YES];

            if !params.filters.is_empty() {
                add_filters(panel, &mut params);
            }

            if let Some(path) = &params.starting_directory {
                set_path(panel, path);
            }

            let res: i32 = msg_send![panel, runModal];

            if res == 1 {
                Some(get_result(panel))
            } else {
                None
            }
        };

        let _: () = msg_send![key_window, makeKeyAndOrderFront: nil];

        pool.drain();

        res
    }
}

pub fn save_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let key_window = key_window();

        let _policy_manager = AppPolicyManager::new();

        let res = {
            let panel = save_panel();

            let _: () = msg_send![panel, setLevel: CGShieldingWindowLevel()];

            // Save filters are unsupported on macos
            //if !params.filters.is_empty() {
            //add_filters(panel, &params);
            //}

            if let Some(path) = &params.starting_directory {
                set_path(panel, path);
            }

            let res: i32 = msg_send![panel, runModal];

            if res == 1 {
                Some(get_result(panel))
            } else {
                None
            }
        };

        let _: () = msg_send![key_window, makeKeyAndOrderFront: nil];

        pool.drain();

        res
    }
}

pub fn pick_folder<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let key_window = key_window();

        let _policy_manager = AppPolicyManager::new();

        let res = {
            let panel = open_panel();

            let _: () = msg_send![panel, setLevel: CGShieldingWindowLevel()];

            let _: () = msg_send![panel, setCanChooseDirectories: YES];
            let _: () = msg_send![panel, setCanChooseFiles: NO];

            if let Some(path) = &params.starting_directory {
                set_path(panel, path);
            }

            let res: i32 = msg_send![panel, runModal];

            if res == 1 {
                Some(get_result(panel))
            } else {
                None
            }
        };

        let _: () = msg_send![key_window, makeKeyAndOrderFront: nil];

        pool.drain();

        res
    }
}

pub fn pick_files<'a>(mut params: impl Into<Option<DialogOptions<'a>>>) -> Option<Vec<PathBuf>> {
    let params = params.into().unwrap_or_default();

    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let key_window = key_window();

        let _policy_manager = AppPolicyManager::new();

        let res = {
            let panel = open_panel();

            let _: () = msg_send![panel, setLevel: CGShieldingWindowLevel()];

            let _: () = msg_send![panel, setCanChooseDirectories: NO];
            let _: () = msg_send![panel, setCanChooseFiles: YES];
            let _: () = msg_send![panel, setAllowsMultipleSelection: YES];

            if !params.filters.is_empty() {
                add_filters(panel, &mut params);
            }

            if let Some(path) = &params.starting_directory {
                set_path(panel, path);
            }

            let res: i32 = msg_send![panel, runModal];

            if res == 1 {
                Some(get_results(panel))
            } else {
                None
            }
        };

        let _: () = msg_send![key_window, makeKeyAndOrderFront: nil];

        pool.drain();

        res
    }
}
