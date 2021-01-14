use crate::FileDialog;
use crate::FileHandle;

use gtk_sys::GtkFileChooser;
use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    ptr,
};

pub fn pick_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    if !gtk_init_check() {
        return None;
    };

    let mut dialog = GtkDialog::new("Open File", GtkFileChooserAction::Open, "Cancel", "Open");

    dialog.add_filters(&opt.filters);
    dialog.set_path(&opt.starting_directory);

    let out = if dialog.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
        dialog.get_result()
    } else {
        None
    };

    dialog.destroy();

    out
}

pub fn save_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    if !gtk_init_check() {
        return None;
    };

    let mut dialog = GtkDialog::new("Save File", GtkFileChooserAction::Save, "Cancel", "Save");

    unsafe { gtk_sys::gtk_file_chooser_set_do_overwrite_confirmation(dialog.ptr, 1) };

    dialog.add_filters(&opt.filters);
    dialog.set_path(&opt.starting_directory);

    let out = if dialog.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
        dialog.get_result()
    } else {
        None
    };

    dialog.destroy();

    out
}

pub fn pick_folder<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    if !gtk_init_check() {
        return None;
    };

    let dialog = GtkDialog::new(
        "Select Folder",
        GtkFileChooserAction::SelectFolder,
        "Cancel",
        "Select",
    );

    dialog.set_path(&opt.starting_directory);

    let out = if dialog.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
        dialog.get_result()
    } else {
        None
    };

    dialog.destroy();

    out
}

pub fn pick_files<'a>(opt: &FileDialog<'a>) -> Option<Vec<PathBuf>> {
    if !gtk_init_check() {
        return None;
    };

    let mut dialog = GtkDialog::new("Open File", GtkFileChooserAction::Open, "Cancel", "Open");

    unsafe { gtk_sys::gtk_file_chooser_set_select_multiple(dialog.ptr, 1) };

    dialog.add_filters(&opt.filters);
    dialog.set_path(&opt.starting_directory);

    let out = if dialog.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
        Some(dialog.get_results())
    } else {
        None
    };

    dialog.destroy();

    out
}

//
//
//

use std::future::Future;

pub fn pick_file_async<'a>(opt: &FileDialog<'a>) -> impl Future<Output = Option<FileHandle>> {
    unimplemented!("");
}

pub fn save_file_async<'a>(opt: &FileDialog<'a>) -> impl Future<Output = Option<FileHandle>> {
    unimplemented!("");
}

pub fn pick_folder_async<'a>(opt: &FileDialog<'a>) -> impl Future<Output = Option<FileHandle>> {
    unimplemented!("");
}

pub fn pick_files_async<'a>(opt: &FileDialog<'a>) -> impl Future<Output = Option<Vec<FileHandle>>> {
    unimplemented!("");
}

//
// Internal
//

fn gtk_init_check() -> bool {
    unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1 }
}

/// gtk_main_iteration()
pub unsafe fn wait_for_cleanup() {
    while gtk_sys::gtk_events_pending() == 1 {
        gtk_sys::gtk_main_iteration();
    }
}

#[repr(i32)]
pub enum GtkFileChooserAction {
    Open = 0,
    Save = 1,
    SelectFolder = 2,
    // CreateFolder = 3,
}

struct GtkDialog {
    ptr: *mut GtkFileChooser,
}

impl GtkDialog {
    fn new(title: &str, action: GtkFileChooserAction, btn1: &str, btn2: &str) -> Self {
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

    pub fn add_filters(&mut self, filters: &[crate::Filter]) {
        for f in filters.iter() {
            if let Ok(name) = CString::new(f.name) {
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

    pub fn set_path(&self, path: &Option<&Path>) {
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

    pub fn get_result(&self) -> Option<PathBuf> {
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

    pub fn get_results(&self) -> Vec<PathBuf> {
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

    pub fn run(&self) -> i32 {
        unsafe { gtk_sys::gtk_dialog_run(self.ptr as *mut _) }
    }

    pub fn destroy(self) {
        unsafe {
            wait_for_cleanup();
            gtk_sys::gtk_widget_destroy(self.ptr as *mut _);
            wait_for_cleanup();
        }
    }
}

//
// ASYNC
//

/*
unsafe fn connect_raw<F>(
    receiver: *mut gobject_sys::GObject,
    signal_name: *const c_char,
    trampoline: GCallback,
    closure: *mut F,
) {
    use std::mem;

    use glib_sys::gpointer;

    unsafe extern "C" fn destroy_closure<F>(ptr: *mut c_void, _: *mut gobject_sys::GClosure) {
        // destroy
        Box::<F>::from_raw(ptr as *mut _);
    }
    assert_eq!(mem::size_of::<*mut F>(), mem::size_of::<gpointer>());
    assert!(trampoline.is_some());
    let handle = gobject_sys::g_signal_connect_data(
        receiver,
        signal_name,
        trampoline,
        closure as *mut _,
        Some(destroy_closure::<F>),
        0,
    );
    assert!(handle > 0);
}

pub unsafe fn connect_response<F: Fn(GtkResponseType) + 'static>(
    dialog: *mut GtkFileChooser,
    f: F,
) {
    use std::mem::transmute;

    unsafe extern "C" fn response_trampoline<F: Fn(GtkResponseType) + 'static>(
        this: *mut gtk_sys::GtkDialog,
        res: GtkResponseType,
        f: glib_sys::gpointer,
    ) {
        let f: &F = &*(f as *const F);

        f(res);
    }
    let f: Box<F> = Box::new(f);
    connect_raw(
        dialog as *mut _,
        b"response\0".as_ptr() as *const _,
        Some(transmute::<_, unsafe extern "C" fn()>(
            response_trampoline::<F> as *const (),
        )),
        Box::into_raw(f),
    );
}
*/
