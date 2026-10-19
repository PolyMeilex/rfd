use super::super::AsGtkDialog;
use crate::FileDialog;
use gtk_sys::GtkFileChooserNative;

use glib::prelude::*;
use glib::translate::{from_glib_full, FromGlibPtrContainer, IntoGlib};
use glib::GString;

use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

#[repr(i32)]
pub enum GtkFileChooserAction {
    Open = 0,
    Save = 1,
    SelectFolder = 2,
    // CreateFolder = 3,
}

pub struct GtkFileDialog {
    object: glib::Object,
}

impl GtkFileDialog {
    fn new(title: &str, action: GtkFileChooserAction, parent: Option<&glib::Object>) -> Self {
        let title = GString::from_string_checked(title.to_owned()).unwrap();
        let parent = parent.map_or(std::ptr::null_mut(), |p| {
            p.as_ptr() as *mut gtk_sys::GtkWindow
        });

        let object = unsafe {
            let dialog = gtk_sys::gtk_file_chooser_native_new(
                title.as_ptr(),
                parent,
                action as i32,
                // passing null for the texts will use the default text, which has full support for i18n
                std::ptr::null(),
                std::ptr::null(),
            );
            // GtkNativeDialog is a plain GObject, we own the returned reference
            from_glib_full(dialog as *mut glib::gobject_ffi::GObject)
        };

        Self { object }
    }

    fn ptr(&self) -> *mut GtkFileChooserNative {
        self.object.as_ptr() as *mut _
    }

    fn add_filters(&mut self, filters: &[crate::file_dialog::Filter]) {
        for f in filters.iter() {
            if let Ok(name) = GString::from_string_checked(f.name.clone()) {
                unsafe {
                    let filter = gtk_sys::gtk_file_filter_new();

                    let paterns: Vec<_> = f
                        .extensions
                        .iter()
                        .filter_map(|e| {
                            if e == "*" || e.is_empty() {
                                Some(GString::from("*"))
                            } else {
                                GString::from_string_checked(format!("*.{}", e)).ok()
                            }
                        })
                        .collect();

                    gtk_sys::gtk_file_filter_set_name(filter, name.as_ptr());

                    for p in paterns.iter() {
                        gtk_sys::gtk_file_filter_add_pattern(filter, p.as_ptr());
                    }

                    gtk_sys::gtk_file_chooser_add_filter(self.ptr() as _, filter);
                }
            }
        }
    }

    fn set_file_name(&self, name: Option<&str>) {
        if let Some(name) = name {
            if let Ok(name) = GString::from_string_checked(name.to_owned()) {
                unsafe {
                    gtk_sys::gtk_file_chooser_set_filename(self.ptr() as _, name.as_ptr());
                }
            }
        }
    }

    fn set_current_name(&self, name: Option<&str>) {
        if let Some(name) = name {
            if let Ok(name) = GString::from_string_checked(name.to_owned()) {
                unsafe {
                    gtk_sys::gtk_file_chooser_set_current_name(self.ptr() as _, name.as_ptr());
                }
            }
        }
    }

    fn set_path(&self, path: Option<&Path>) {
        if let Some(path) = path {
            if let Ok(path) = GString::try_from(path.to_owned()) {
                unsafe {
                    gtk_sys::gtk_file_chooser_set_current_folder(self.ptr() as _, path.as_ptr());
                }
            }
        }
    }

    fn set_show_hidden(&self, show: Option<bool>) {
        if let Some(show) = show {
            unsafe {
                gtk_sys::gtk_file_chooser_set_show_hidden(self.ptr() as _, show.into_glib());
            }
        }
    }

    pub fn get_result(&self) -> Option<PathBuf> {
        unsafe { from_glib_full(gtk_sys::gtk_file_chooser_get_filename(self.ptr() as _)) }
    }

    pub fn get_results(&self) -> Vec<PathBuf> {
        unsafe {
            FromGlibPtrContainer::from_glib_full(gtk_sys::gtk_file_chooser_get_filenames(
                self.ptr() as _,
            ))
        }
    }

    pub fn run(&self) -> i32 {
        unsafe { gtk_sys::gtk_native_dialog_run(self.ptr() as *mut _) }
    }
}

fn parent_gtk_window(opt: &FileDialog) -> Option<glib::Object> {
    let parent_handle = opt.parent.as_ref()?;
    unsafe { super::super::utils::find_gtk_window(parent_handle) }
}

impl GtkFileDialog {
    pub fn build_pick_file(opt: &FileDialog) -> Self {
        let mut dialog = GtkFileDialog::new(
            opt.title.as_deref().unwrap_or("Open File"),
            GtkFileChooserAction::Open,
            parent_gtk_window(opt).as_ref(),
        );

        dialog.add_filters(&opt.filters);
        dialog.set_path(opt.starting_directory.as_deref());
        dialog.set_show_hidden(opt.show_hidden_files);

        if let (Some(mut path), Some(file_name)) =
            (opt.starting_directory.to_owned(), opt.file_name.as_deref())
        {
            path.push(file_name);
            dialog.set_file_name(path.deref().to_str());
        } else {
            dialog.set_file_name(opt.file_name.as_deref());
        }

        dialog
    }

    pub fn build_save_file(opt: &FileDialog) -> Self {
        let mut dialog = GtkFileDialog::new(
            opt.title.as_deref().unwrap_or("Save File"),
            GtkFileChooserAction::Save,
            parent_gtk_window(opt).as_ref(),
        );

        unsafe { gtk_sys::gtk_file_chooser_set_do_overwrite_confirmation(dialog.ptr() as _, 1) };

        dialog.add_filters(&opt.filters);
        dialog.set_path(opt.starting_directory.as_deref());
        dialog.set_show_hidden(opt.show_hidden_files);

        if let (Some(mut path), Some(file_name)) =
            (opt.starting_directory.to_owned(), opt.file_name.as_deref())
        {
            path.push(file_name);
            if path.exists() {
                // the user edited an existing document
                dialog.set_file_name(path.deref().to_str());
            } else {
                // the user just created a new document
                dialog.set_current_name(opt.file_name.as_deref());
            }
        } else {
            // the user just created a new document
            dialog.set_current_name(opt.file_name.as_deref());
        }

        dialog
    }

    pub fn build_pick_folder(opt: &FileDialog) -> Self {
        let dialog = GtkFileDialog::new(
            opt.title.as_deref().unwrap_or("Select Folder"),
            GtkFileChooserAction::SelectFolder,
            parent_gtk_window(opt).as_ref(),
        );
        dialog.set_path(opt.starting_directory.as_deref());
        dialog.set_show_hidden(opt.show_hidden_files);

        if let (Some(mut path), Some(file_name)) =
            (opt.starting_directory.to_owned(), opt.file_name.as_deref())
        {
            path.push(file_name);
            dialog.set_file_name(path.deref().to_str());
        } else {
            dialog.set_file_name(opt.file_name.as_deref());
        }

        dialog
    }

    pub fn build_pick_folders(opt: &FileDialog) -> Self {
        let dialog = GtkFileDialog::new(
            opt.title.as_deref().unwrap_or("Select Folder"),
            GtkFileChooserAction::SelectFolder,
            parent_gtk_window(opt).as_ref(),
        );
        unsafe { gtk_sys::gtk_file_chooser_set_select_multiple(dialog.ptr() as _, 1) };
        dialog.set_path(opt.starting_directory.as_deref());
        dialog.set_show_hidden(opt.show_hidden_files);

        if let (Some(mut path), Some(file_name)) =
            (opt.starting_directory.to_owned(), opt.file_name.as_deref())
        {
            path.push(file_name);
            dialog.set_file_name(path.deref().to_str());
        } else {
            dialog.set_file_name(opt.file_name.as_deref());
        }

        dialog
    }

    pub fn build_pick_files(opt: &FileDialog) -> Self {
        let mut dialog = GtkFileDialog::new(
            opt.title.as_deref().unwrap_or("Open File"),
            GtkFileChooserAction::Open,
            parent_gtk_window(opt).as_ref(),
        );

        unsafe { gtk_sys::gtk_file_chooser_set_select_multiple(dialog.ptr() as _, 1) };
        dialog.add_filters(&opt.filters);
        dialog.set_path(opt.starting_directory.as_deref());
        dialog.set_show_hidden(opt.show_hidden_files);

        if let (Some(mut path), Some(file_name)) =
            (opt.starting_directory.to_owned(), opt.file_name.as_deref())
        {
            path.push(file_name);
            dialog.set_file_name(path.deref().to_str());
        } else {
            dialog.set_file_name(opt.file_name.as_deref());
        }

        dialog
    }
}

impl AsGtkDialog for GtkFileDialog {
    fn object(&self) -> &glib::Object {
        &self.object
    }

    unsafe fn show(&self) {
        gtk_sys::gtk_native_dialog_show(self.ptr() as *mut _);
    }
}

impl Drop for GtkFileDialog {
    fn drop(&mut self) {
        unsafe {
            gtk_sys::gtk_native_dialog_destroy(self.ptr() as _);
        }
    }
}
