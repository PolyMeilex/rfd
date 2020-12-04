use crate::DialogParams;
use std::{ffi::CStr, path::PathBuf, ptr};

use gtk_sys::GtkFileChooser;

#[repr(i32)]
enum GtkFileChooserAction {
    Open = 0,
    Save = 1,
    SelectFolder = 2,
    CreateFolder = 3,
}

unsafe fn build_gtk_dialog(
    title: &str,
    action: GtkFileChooserAction,
    btn1: &str,
    btn2: &str,
) -> *mut GtkFileChooser {
    let dialog = gtk_sys::gtk_file_chooser_dialog_new(
        title.as_ptr() as *const _,
        ptr::null_mut(),
        action as i32,
        btn1.as_ptr() as *const _,
        gtk_sys::GTK_RESPONSE_CANCEL,
        btn2.as_ptr() as *const _,
        gtk_sys::GTK_RESPONSE_ACCEPT,
        ptr::null_mut::<i8>(),
    );
    dialog as _
}

unsafe fn add_filters(dialog: *mut GtkFileChooser, filters: &[(&str, &str)]) {
    for f in filters.iter() {
        let filter = gtk_sys::gtk_file_filter_new();

        let name = format!("{}\0", f.0);
        let pat = format!("{}\0", f.1);

        gtk_sys::gtk_file_filter_set_name(filter, name.as_ptr() as *const _);
        gtk_sys::gtk_file_filter_add_pattern(filter, pat.as_ptr() as *const _);

        gtk_sys::gtk_file_chooser_add_filter(dialog, filter);
    }
}

/// gtk_init_check()
unsafe fn init_check() -> bool {
    gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1
}

unsafe fn wait_for_cleanup() {
    while gtk_sys::gtk_events_pending() == 1 {
        gtk_sys::gtk_main_iteration();
    }
}

pub fn open_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        let gtk_inited = init_check();

        if gtk_inited {
            let dialog = build_gtk_dialog(
                "Open File\0",
                GtkFileChooserAction::Open,
                "Cancel\0",
                "Open\0",
            );

            add_filters(dialog, &params.filters);

            let res = gtk_sys::gtk_dialog_run(dialog as *mut _);

            let out = if res == gtk_sys::GTK_RESPONSE_ACCEPT {
                let chosen_filename = gtk_sys::gtk_file_chooser_get_filename(dialog as *mut _);

                let cstr = CStr::from_ptr(chosen_filename).to_str();

                if let Ok(cstr) = cstr {
                    Some(PathBuf::from(cstr.to_owned()))
                } else {
                    None
                }
            } else {
                None
            };

            wait_for_cleanup();
            gtk_sys::gtk_widget_destroy(dialog as *mut _);
            wait_for_cleanup();

            out
        } else {
            None
        }
    }
}

pub fn open_multiple_files_with_params(params: DialogParams) -> Option<Vec<PathBuf>> {
    unimplemented!("open_multiple_with_params");
}
