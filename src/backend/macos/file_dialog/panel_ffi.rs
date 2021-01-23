use crate::FileDialog;

use std::path::Path;
use std::path::PathBuf;

use objc::{class, msg_send, sel, sel_impl};

use cocoa_foundation::base::id;
use cocoa_foundation::base::nil;
use cocoa_foundation::foundation::{NSArray, NSAutoreleasePool, NSString, NSURL};
use objc::runtime::{Object, YES};
use objc::runtime::{BOOL, NO};

use super::super::policy_manager::PolicyManager;
use objc::rc::StrongPtr;

use super::super::AsModal;

extern "C" {
    pub fn CGShieldingWindowLevel() -> i32;
}

fn make_nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s).autorelease() }
}

pub struct Panel {
    pub(crate) panel: StrongPtr,
    _policy_manager: PolicyManager,
    key_window: *mut Object,
}

impl AsModal for Panel {
    fn modal_ptr(&self) -> id {
        *self.panel
    }
}

impl Panel {
    pub fn new(panel: *mut Object) -> Self {
        let _policy_manager = PolicyManager::new();
        let key_window = unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            msg_send![app, keyWindow]
        };

        let _: () = unsafe { msg_send![panel, setLevel: CGShieldingWindowLevel()] };
        Self {
            _policy_manager,
            panel: unsafe { StrongPtr::retain(panel) },
            key_window,
        }
    }

    pub fn open_panel() -> Self {
        Self::new(unsafe { msg_send![class!(NSOpenPanel), openPanel] })
    }

    pub fn save_panel() -> Self {
        Self::new(unsafe { msg_send![class!(NSSavePanel), savePanel] })
    }

    pub fn run_modal(&self) -> i32 {
        unsafe { msg_send![*self.panel, runModal] }
    }

    pub fn set_can_choose_directories(&self, v: BOOL) {
        let _: () = unsafe { msg_send![*self.panel, setCanChooseDirectories: v] };
    }

    pub fn set_can_choose_files(&self, v: BOOL) {
        let _: () = unsafe { msg_send![*self.panel, setCanChooseFiles: v] };
    }

    pub fn set_allows_multiple_selection(&self, v: BOOL) {
        let _: () = unsafe { msg_send![*self.panel, setAllowsMultipleSelection: v] };
    }

    pub fn add_filters(&self, params: &FileDialog) {
        let mut exts: Vec<String> = Vec::new();

        for filter in params.filters.iter() {
            exts.append(&mut filter.extensions.to_vec());
        }

        unsafe {
            let f_raw: Vec<_> = exts.iter().map(|ext| make_nsstring(&ext)).collect();

            let array = NSArray::arrayWithObjects(nil, f_raw.as_slice());
            let _: () = msg_send![*self.panel, setAllowedFileTypes: array];
        }
    }

    pub fn set_path(&self, path: &Path) {
        if let Some(path) = path.to_str() {
            unsafe {
                let url = NSURL::alloc(nil)
                    .initFileURLWithPath_isDirectory_(make_nsstring(path), YES)
                    .autorelease();
                let () = msg_send![*self.panel, setDirectoryURL: url];
            }
        }
    }

    pub fn get_result(&self) -> PathBuf {
        unsafe {
            let url: id = msg_send![*self.panel, URL];
            let path: id = msg_send![url, path];
            let utf8: *const i32 = msg_send![path, UTF8String];
            let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

            let slice = std::slice::from_raw_parts(utf8 as *const _, len);
            let result = std::str::from_utf8_unchecked(slice);

            result.into()
        }
    }

    pub fn get_results(&self) -> Vec<PathBuf> {
        unsafe {
            let urls: id = msg_send![*self.panel, URLs];

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

impl Panel {
    pub fn build_pick_file(opt: &FileDialog) -> Self {
        let panel = Panel::open_panel();

        if !opt.filters.is_empty() {
            panel.add_filters(&opt);
        }

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        panel.set_can_choose_directories(NO);
        panel.set_can_choose_files(YES);

        panel
    }

    pub fn build_save_file(opt: &FileDialog) -> Self {
        let panel = Panel::save_panel();

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        panel
    }

    pub fn build_pick_folder(opt: &FileDialog) -> Self {
        let panel = Panel::open_panel();

        if let Some(path) = &opt.starting_directory {
            panel.set_path(path);
        }

        panel.set_can_choose_directories(YES);
        panel.set_can_choose_files(NO);

        panel
    }

    pub fn build_pick_files(opt: &FileDialog) -> Self {
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

        panel
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.key_window, makeKeyAndOrderFront: nil] };
    }
}
