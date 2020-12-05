use crate::DialogParams;
use std::path::PathBuf;

use objc::runtime::Object;
use objc::{class, msg_send, sel, sel_impl};

use cocoa_foundation::base::{id, nil};
use cocoa_foundation::foundation::{NSArray, NSAutoreleasePool};
pub use objc::runtime::{BOOL, NO, YES};

mod utils {
    use cocoa_foundation::base::{id, nil};
    use cocoa_foundation::foundation::{NSAutoreleasePool, NSString};
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

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

    extern "C" {
        pub fn CGShieldingWindowLevel() -> i32;
    }

    #[repr(i32)]
    #[derive(Debug, PartialEq)]
    pub enum ApplicationActivationPolicy {
        Regular = 0,
        Accessory = 1,
        Prohibited = 2,
        Error = -1,
    }

    pub fn make_nsstring(s: &str) -> id {
        unsafe { NSString::alloc(nil).init_str(s).autorelease() }
    }
}

use utils::*;

pub fn open_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        let app = app();
        let key_window = key_window();

        let prev_policy = {
            let pol: i32 = msg_send![app, activationPolicy];

            if pol == ApplicationActivationPolicy::Prohibited as i32 {
                let new_pol = ApplicationActivationPolicy::Accessory as i32;
                let _: () = msg_send![app, setActivationPolicy: new_pol];
            }
            pol
        };

        let res = {
            let panel = open_panel();

            let level = CGShieldingWindowLevel();
            let _: () = msg_send![panel, setLevel: level];

            let _: () = msg_send![panel, setCanChooseDirectories: YES];
            let _: () = msg_send![panel, setCanChooseFiles: YES];

            if !params.filters.is_empty() {
                let new_filters: Vec<String> = params
                    .filters
                    .iter()
                    .map(|(_, ext)| ext.to_string().replace("*.", ""))
                    .collect();

                let f_raw: Vec<_> = new_filters.iter().map(|ext| make_nsstring(ext)).collect();

                let array_raw = NSArray::arrayWithObjects(nil, f_raw.as_slice());

                let _: () = msg_send![panel, setAllowedFileTypes: array_raw];
            }

            let res: i32 = msg_send![panel, runModal];

            if res == 1 {
                let url: id = msg_send![panel, URL];
                let path: id = msg_send![url, path];
                let utf8: *const i32 = msg_send![path, UTF8String];
                let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

                let slice = std::slice::from_raw_parts(utf8 as *const _, len);
                let result = std::str::from_utf8_unchecked(slice);

                Some(result.into())
            } else {
                None
            }
        };

        let _: () = msg_send![key_window, makeKeyAndOrderFront: nil];
        let _: () = msg_send![app, setActivationPolicy: prev_policy];

        pool.drain();

        res
    }
}

pub fn save_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unimplemented!("save_file_with_params");
}

pub fn pick_folder() -> Option<PathBuf> {
    unimplemented!("pick_folder");
}

pub fn open_multiple_files_with_params(params: DialogParams) -> Option<Vec<PathBuf>> {
    unimplemented!("open_multiple_with_params");
}
