use crate::DialogParams;
use std::{ffi::CStr, path::PathBuf};

mod utils {
    use gtk_sys::GtkFileChooser;
    use std::ptr;

    #[repr(i32)]
    pub enum GtkFileChooserAction {
        Open = 0,
        Save = 1,
        SelectFolder = 2,
        // CreateFolder = 3,
    }

    pub unsafe fn build_gtk_dialog(
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

    pub unsafe fn add_filters(dialog: *mut GtkFileChooser, filters: &[(&str, &str)]) {
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
    pub unsafe fn init_check() -> bool {
        gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1
    }

    pub unsafe fn wait_for_cleanup() {
        while gtk_sys::gtk_events_pending() == 1 {
            gtk_sys::gtk_main_iteration();
        }
    }
}

use utils::*;

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

pub fn save_file_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        let gtk_inited = init_check();

        if gtk_inited {
            let dialog = build_gtk_dialog(
                "Save File\0",
                GtkFileChooserAction::Save,
                "Cancel\0",
                "Save\0",
            );

            gtk_sys::gtk_file_chooser_set_do_overwrite_confirmation(dialog, 1);

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

pub fn pick_folder_with_params(params: DialogParams) -> Option<PathBuf> {
    unsafe {
        let gtk_inited = init_check();

        if gtk_inited {
            let dialog = build_gtk_dialog(
                "Select Folder\0",
                GtkFileChooserAction::SelectFolder,
                "Cancel\0",
                "Select\0",
            );

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

    unsafe {
        let gtk_inited = init_check();

        if gtk_inited {
            let dialog = build_gtk_dialog(
                "Open File\0",
                GtkFileChooserAction::Open,
                "Cancel\0",
                "Open\0",
            );

            gtk_sys::gtk_file_chooser_set_select_multiple(dialog, 1);

            add_filters(dialog, &params.filters);

            let res = gtk_sys::gtk_dialog_run(dialog as *mut _);

            let out = if res == gtk_sys::GTK_RESPONSE_ACCEPT {
                let chosen_filenames = gtk_sys::gtk_file_chooser_get_filenames(dialog as *mut _);

                let paths: Vec<PathBuf> = FileList(chosen_filenames)
                    .filter_map(|item| {
                        let cstr = CStr::from_ptr(item.data as _).to_str();

                        if let Ok(cstr) = cstr {
                            Some(PathBuf::from(cstr.to_owned()))
                        } else {
                            None
                        }
                    })
                    .collect();

                Some(paths)
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
