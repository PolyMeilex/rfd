use crate::{FileDialog, FileHandle};
use gtk_sys::GtkFileChooser;

use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    ptr,
};

#[repr(i32)]
pub enum GtkFileChooserAction {
    Open = 0,
    Save = 1,
    SelectFolder = 2,
    // CreateFolder = 3,
}

pub(super) struct GtkDialog {
    pub ptr: *mut GtkFileChooser,
}

impl GtkDialog {
    pub(super) fn new(title: &str, action: GtkFileChooserAction, btn1: &str, btn2: &str) -> Self {
        let title = CString::new(title).unwrap();
        let btn1 = CString::new(btn1).unwrap();
        let btn2 = CString::new(btn2).unwrap();

        let ptr = unsafe {
            let dialog = gtk_sys::gtk_file_chooser_dialog_new(
                title.as_ptr(),
                ptr::null_mut(),
                action as i32,
                btn1.as_ptr(),
                gtk_sys::GTK_RESPONSE_CANCEL,
                btn2.as_ptr(),
                gtk_sys::GTK_RESPONSE_ACCEPT,
                ptr::null_mut::<i8>(),
            );
            dialog as _
        };

        Self { ptr }
    }

    pub(super) fn add_filters(&mut self, filters: &[crate::dialog::Filter]) {
        for f in filters.iter() {
            if let Ok(name) = CString::new(f.name.as_str()) {
                unsafe {
                    let filter = gtk_sys::gtk_file_filter_new();

                    let paterns: Vec<_> = f
                        .extensions
                        .iter()
                        .filter_map(|e| CString::new(format!("*.{}", e)).ok())
                        .collect();

                    gtk_sys::gtk_file_filter_set_name(filter, name.as_ptr());

                    for p in paterns.iter() {
                        gtk_sys::gtk_file_filter_add_pattern(filter, p.as_ptr());
                    }

                    gtk_sys::gtk_file_chooser_add_filter(self.ptr, filter);
                }
            }
        }
    }

    pub(super) fn set_path(&self, path: &Option<PathBuf>) {
        if let Some(path) = path {
            if let Some(path) = path.to_str() {
                if let Ok(path) = CString::new(path) {
                    unsafe {
                        gtk_sys::gtk_file_chooser_set_current_folder(self.ptr, path.as_ptr());
                    }
                }
            }
        }
    }

    pub(super) fn get_result(&self) -> Option<PathBuf> {
        let cstr = unsafe {
            let chosen_filename = gtk_sys::gtk_file_chooser_get_filename(self.ptr as *mut _);
            CStr::from_ptr(chosen_filename).to_str()
        };

        if let Ok(cstr) = cstr {
            Some(PathBuf::from(cstr.to_owned()))
        } else {
            None
        }
    }

    pub(super) fn get_results(&self) -> Vec<PathBuf> {
        #[derive(Debug)]
        struct FileList(*mut glib_sys::GSList);

        impl Iterator for FileList {
            type Item = glib_sys::GSList;
            fn next(&mut self) -> Option<Self::Item> {
                let curr_ptr = self.0;

                if !curr_ptr.is_null() {
                    let curr = unsafe { *curr_ptr };

                    self.0 = curr.next;

                    Some(curr)
                } else {
                    None
                }
            }
        }

        let chosen_filenames =
            unsafe { gtk_sys::gtk_file_chooser_get_filenames(self.ptr as *mut _) };

        let paths: Vec<PathBuf> = FileList(chosen_filenames)
            .filter_map(|item| {
                let cstr = unsafe { CStr::from_ptr(item.data as _).to_str() };

                if let Ok(cstr) = cstr {
                    Some(PathBuf::from(cstr.to_owned()))
                } else {
                    None
                }
            })
            .collect();

        paths
    }

    pub(super) fn run(&self) -> i32 {
        unsafe { gtk_sys::gtk_dialog_run(self.ptr as *mut _) }
    }
}

impl GtkDialog {
    pub fn build_pick_file<'a>(opt: &FileDialog) -> Self {
        let mut dialog = GtkDialog::new("Open File", GtkFileChooserAction::Open, "Cancel", "Open");

        dialog.add_filters(&opt.filters);
        dialog.set_path(&opt.starting_directory);
        dialog
    }

    pub fn build_save_file<'a>(opt: &FileDialog) -> Self {
        let mut dialog = GtkDialog::new("Save File", GtkFileChooserAction::Save, "Cancel", "Save");

        unsafe { gtk_sys::gtk_file_chooser_set_do_overwrite_confirmation(dialog.ptr, 1) };

        dialog.add_filters(&opt.filters);
        dialog.set_path(&opt.starting_directory);
        dialog
    }

    pub fn build_pick_folder<'a>(opt: &FileDialog) -> Self {
        let dialog = GtkDialog::new(
            "Select Folder",
            GtkFileChooserAction::SelectFolder,
            "Cancel",
            "Select",
        );
        dialog.set_path(&opt.starting_directory);
        dialog
    }

    pub fn build_pick_files<'a>(opt: &FileDialog) -> Self {
        let mut dialog = GtkDialog::new("Open File", GtkFileChooserAction::Open, "Cancel", "Open");

        unsafe { gtk_sys::gtk_file_chooser_set_select_multiple(dialog.ptr, 1) };
        dialog.add_filters(&opt.filters);
        dialog.set_path(&opt.starting_directory);
        dialog
    }
}

pub(super) trait OutputFrom<F> {
    fn from(from: &F, res_id: i32) -> Self;
    /// Describes what should be returned when gtk_init failed
    fn get_failed() -> Self;
}

impl OutputFrom<GtkDialog> for Option<PathBuf> {
    fn from(dialog: &GtkDialog, res_id: i32) -> Self {
        if res_id == gtk_sys::GTK_RESPONSE_ACCEPT {
            dialog.get_result()
        } else {
            None
        }
    }
    fn get_failed() -> Self {
        None
    }
}

impl OutputFrom<GtkDialog> for Option<Vec<PathBuf>> {
    fn from(dialog: &GtkDialog, res_id: i32) -> Self {
        if res_id == gtk_sys::GTK_RESPONSE_ACCEPT {
            Some(dialog.get_results())
        } else {
            None
        }
    }
    fn get_failed() -> Self {
        None
    }
}

impl OutputFrom<GtkDialog> for Option<FileHandle> {
    fn from(dialog: &GtkDialog, res_id: i32) -> Self {
        if res_id == gtk_sys::GTK_RESPONSE_ACCEPT {
            dialog.get_result().map(|f| FileHandle::wrap(f))
        } else {
            None
        }
    }
    fn get_failed() -> Self {
        None
    }
}

impl OutputFrom<GtkDialog> for Option<Vec<FileHandle>> {
    fn from(dialog: &GtkDialog, res_id: i32) -> Self {
        if res_id == gtk_sys::GTK_RESPONSE_ACCEPT {
            let files = dialog
                .get_results()
                .into_iter()
                .map(|f| FileHandle::wrap(f))
                .collect();
            Some(files)
        } else {
            None
        }
    }
    fn get_failed() -> Self {
        None
    }
}

impl Drop for GtkDialog {
    fn drop(&mut self) {
        unsafe {
            wait_for_cleanup();
            gtk_sys::gtk_widget_destroy(self.ptr as *mut _);
            wait_for_cleanup();
        }
    }
}

/// gtk_main_iteration()
pub(crate) unsafe fn wait_for_cleanup() {
    while gtk_sys::gtk_events_pending() == 1 {
        gtk_sys::gtk_main_iteration();
    }
}
