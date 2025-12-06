use std::{
    ffi::{CStr, CString},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    ffi,
    libdbus::{Message, MessageIter},
};

#[derive(Debug)]
pub struct HandleToken(pub CString);

impl Default for HandleToken {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let token = format!("rfd_{now}");
        Self(CString::new(token).unwrap())
    }
}

pub type FileFilter = (CString, Vec<CString>);

#[derive(Debug)]
pub struct FilePath(pub CString);

#[derive(Debug, Default)]
pub struct OpenFileOptions {
    pub parent_window: CString,
    pub title: CString,
    pub handle_token: HandleToken,
    pub accept_label: Option<CString>,
    pub modal: Option<bool>,
    pub multiple: Option<bool>,
    pub directory: Option<bool>,
    pub filters: Vec<FileFilter>,
    // TODO: Serialize this if needed
    #[allow(unused)]
    pub current_filter: Option<FileFilter>,
    pub current_folder: Option<FilePath>,
}

fn append_filters(dict: &mut MessageIter, filters: Vec<FileFilter>) {
    let filters: Vec<_> = filters
        .into_iter()
        .filter(|(_, globs)| !globs.is_empty())
        .collect();

    if !filters.is_empty() {
        dict.with_dict_entry(c"filters", c"a(sa(us))", |variant| {
            variant.with_container(ffi::DBUS_TYPE_ARRAY, Some(c"(sa(us))"), |array| {
                for (label, globs) in filters.iter() {
                    array.with_container(ffi::DBUS_TYPE_STRUCT, None, |s| {
                        s.append_string(label);
                        s.with_container(ffi::DBUS_TYPE_ARRAY, Some(c"(us)"), |array| {
                            for glob in globs {
                                array.with_container(ffi::DBUS_TYPE_STRUCT, None, |s| {
                                    s.append_u32(0); // Glob type
                                    s.append_string(glob);
                                })
                            }
                        })
                    });
                }
            });
        });
    }
}

fn append_path(dict: &mut MessageIter, key: &CStr, path: &FilePath) {
    dict.with_dict_entry(key, c"ay", |variant| {
        variant.with_container(ffi::DBUS_TYPE_ARRAY, Some(c"y"), |array| {
            for byte in path.0.as_bytes_with_nul() {
                array.append_byte(*byte);
            }
        })
    });
}

impl Message {
    pub fn open_file(opts: OpenFileOptions) -> Self {
        let mut msg = Message::new_method_call(
            c"org.freedesktop.portal.Desktop",
            c"/org/freedesktop/portal/desktop",
            c"org.freedesktop.portal.FileChooser",
            c"OpenFile",
        )
        .unwrap();

        let mut iter = MessageIter::init_append(&mut msg);

        iter.append_string(&opts.parent_window);
        iter.append_string(&opts.title);

        iter.with_container(ffi::DBUS_TYPE_ARRAY, Some(c"{sv}"), |dict| {
            dict.with_dict_entry(c"handle_token", c"s", |variant| {
                variant.append_string(&opts.handle_token.0);
            });

            if let Some(accept_label) = opts.accept_label.as_ref() {
                dict.with_dict_entry(c"accept_label", c"s", |variant| {
                    variant.append_string(accept_label);
                });
            }

            if let Some(modal) = opts.modal {
                dict.with_dict_entry(c"modal", c"b", |variant| {
                    variant.append_bool(modal);
                });
            }
            if let Some(multiple) = opts.multiple {
                dict.with_dict_entry(c"multiple", c"b", |variant| {
                    variant.append_bool(multiple);
                });
            }
            if let Some(directory) = opts.directory {
                dict.with_dict_entry(c"directory", c"b", |variant| {
                    variant.append_bool(directory);
                });
            }

            if let Some(current_folder) = opts.current_folder {
                append_path(dict, c"current_folder", &current_folder);
            }

            append_filters(dict, opts.filters);
        });

        msg
    }
}

#[derive(Debug, Default)]
pub struct SaveFileOptions {
    pub parent_window: CString,
    pub title: CString,
    pub handle_token: HandleToken,
    pub accept_label: Option<CString>,
    pub modal: Option<bool>,
    pub current_name: Option<CString>,
    pub current_folder: Option<FilePath>,
    pub current_file: Option<FilePath>,
    pub filters: Vec<FileFilter>,
    // TODO: Serialize this if needed
    #[allow(unused)]
    pub current_filter: Option<FileFilter>,
}

impl Message {
    pub fn save_file(opts: SaveFileOptions) -> Self {
        let mut msg = Message::new_method_call(
            c"org.freedesktop.portal.Desktop",
            c"/org/freedesktop/portal/desktop",
            c"org.freedesktop.portal.FileChooser",
            c"SaveFile",
        )
        .unwrap();

        let mut iter = MessageIter::init_append(&mut msg);

        iter.append_string(&opts.parent_window);
        iter.append_string(&opts.title);

        iter.with_container(ffi::DBUS_TYPE_ARRAY, Some(c"{sv}"), |dict| {
            dict.with_dict_entry(c"handle_token", c"s", |variant| {
                variant.append_string(&opts.handle_token.0);
            });

            if let Some(accept_label) = opts.accept_label.as_ref() {
                dict.with_dict_entry(c"accept_label", c"s", |variant| {
                    variant.append_string(accept_label);
                });
            }

            if let Some(modal) = opts.modal {
                dict.with_dict_entry(c"modal", c"b", |variant| {
                    variant.append_bool(modal);
                });
            }

            if let Some(current_name) = opts.current_name.as_ref() {
                dict.with_dict_entry(c"current_name", c"s", |variant| {
                    variant.append_string(current_name);
                });
            }

            if let Some(current_folder) = opts.current_folder {
                append_path(dict, c"current_folder", &current_folder);
            }

            if let Some(current_file) = opts.current_file {
                append_path(dict, c"current_file", &current_file);
            }

            append_filters(dict, opts.filters);
        });

        msg
    }
}
