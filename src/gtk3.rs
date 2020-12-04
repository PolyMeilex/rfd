use crate::DialogParams;
use std::{ffi::CStr, path::PathBuf, ptr};

pub fn open_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        let gtk_inited = gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1;

        if gtk_inited {
            let dialog = gtk_sys::gtk_file_chooser_dialog_new(
                "Open File\0".as_ptr() as *const _,
                ptr::null_mut(),
                gtk_sys::GTK_FILE_CHOOSER_ACTION_OPEN,
                "Cancel\0".as_ptr() as *const _,
                gtk_sys::GTK_RESPONSE_CANCEL,
                "Open\0".as_ptr() as *const _,
                gtk_sys::GTK_RESPONSE_ACCEPT,
                ptr::null_mut::<i8>(),
            ) as *mut gtk_sys::GtkFileChooser;

            for f in params.filters.iter() {
                let filter = gtk_sys::gtk_file_filter_new();

                let name = format!("{}\0", f.0);
                let pat = format!("{}\0", f.1);

                gtk_sys::gtk_file_filter_set_name(filter, name.as_ptr() as *const _);
                gtk_sys::gtk_file_filter_add_pattern(filter, pat.as_ptr() as *const _);

                gtk_sys::gtk_file_chooser_add_filter(dialog, filter);
            }

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

fn wait_for_cleanup() {
    unsafe {
        while gtk_sys::gtk_events_pending() == 1 {
            gtk_sys::gtk_main_iteration();
        }
    }
}
