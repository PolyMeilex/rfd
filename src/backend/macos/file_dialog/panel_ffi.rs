use crate::FileDialog;

use std::path::Path;
use std::{ops::DerefMut, path::PathBuf};

use objc::{class, msg_send, sel, sel_impl};
use objc_id::Id;

use super::super::utils::{nil, INSURL, NSURL};

use objc::runtime::{Object, YES};
use objc::runtime::{BOOL, NO};
use objc_foundation::{INSArray, INSString, NSArray, NSString};

use super::super::policy_manager::PolicyManager;

use super::super::AsModal;

extern "C" {
    pub fn CGShieldingWindowLevel() -> i32;
}

fn make_nsstring(s: &str) -> Id<NSString> {
    NSString::from_str(s)
}

pub struct Panel {
    pub(crate) panel: Id<Object>,
    _policy_manager: PolicyManager,
    key_window: *mut Object,
}

impl AsModal for Panel {
    fn modal_ptr(&mut self) -> *mut Object {
        self.panel.deref_mut()
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
            panel: unsafe { Id::from_ptr(panel) },
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
        unsafe { msg_send![self.panel, runModal] }
    }

    pub fn set_can_choose_directories(&self, v: BOOL) {
        let _: () = unsafe { msg_send![self.panel, setCanChooseDirectories: v] };
    }

    pub fn set_can_choose_files(&self, v: BOOL) {
        let _: () = unsafe { msg_send![self.panel, setCanChooseFiles: v] };
    }

    pub fn set_allows_multiple_selection(&self, v: BOOL) {
        let _: () = unsafe { msg_send![self.panel, setAllowsMultipleSelection: v] };
    }

    pub fn add_filters(&self, params: &FileDialog) {
        let mut exts: Vec<String> = Vec::new();

        for filter in params.filters.iter() {
            exts.append(&mut filter.extensions.to_vec());
        }

        unsafe {
            let f_raw: Vec<_> = exts.iter().map(|ext| make_nsstring(&ext)).collect();
            let array = NSArray::from_vec(f_raw);

            let _: () = msg_send![self.panel, setAllowedFileTypes: array];
        }
    }

    pub fn set_path(&self, path: &Path) {
        if let Some(path) = path.to_str() {
            unsafe {
                let url = NSURL::file_url_with_path(path, true);
                let () = msg_send![self.panel, setDirectoryURL: url];
            }
        }
    }

    pub fn get_result(&self) -> PathBuf {
        unsafe {
            let url = msg_send![self.panel, URL];
            let url: Id<NSURL> = Id::from_ptr(url);
            url.to_path_buf()
        }
    }

    pub fn get_results(&self) -> Vec<PathBuf> {
        unsafe {
            let urls = msg_send![self.panel, URLs];
            let urls: Id<NSArray<NSURL>> = Id::from_ptr(urls);

            let mut res = Vec::new();
            for url in urls.to_vec() {
                res.push(url.to_path_buf());
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

        if !opt.filters.is_empty() {
            panel.add_filters(&opt);
        }

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
