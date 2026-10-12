use std::{
    ffi::{CStr, CString, OsStr},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

mod ffi;

mod file_dialog;

mod libdbus;
use libdbus::*;

pub use file_dialog::{FileFilter, FilePath, HandleToken, OpenFileOptions, SaveFileOptions};

fn uris_to_paths(uris: Vec<CString>) -> Vec<PathBuf> {
    uris.into_iter()
        .filter_map(|uri| {
            let bytes: Vec<u8> = percent_encoding::percent_decode(uri.as_bytes()).collect();
            let Some(path) = bytes.strip_prefix(b"file://") else {
                log::error!("Ignoring uri: {bytes:?} lacks `file://`");
                return None;
            };
            Some(PathBuf::from(OsStr::from_bytes(path)))
        })
        .collect()
}

pub fn open_file(opts: OpenFileOptions) -> Option<Response> {
    let mut conn = Connection::new()?;

    let handle_path = generate_response_path(&mut conn, &opts.handle_token);
    register_response_listener(&mut conn, &handle_path);

    let reply = conn
        .send_and_block(&Message::open_file(opts))
        .inspect_err(|err| log::error!("OpenFile failed: {err}"))
        .ok()??;

    let mut iter = MessageIter::from_msg(&reply);

    let Some(got_handle_path) = iter.get_object_path() else {
        log::error!("Response object path is missing");
        return None;
    };

    if handle_path != got_handle_path {
        log::debug!("Detected ancient version of xdg portal, attempting fallback");
        register_response_listener(&mut conn, &got_handle_path);
    }

    wait_for_response(&mut conn, &got_handle_path)
}

pub fn save_file(opts: SaveFileOptions) -> Option<Response> {
    let mut conn = Connection::new()?;

    let handle_path = generate_response_path(&mut conn, &opts.handle_token);
    register_response_listener(&mut conn, &handle_path);

    let reply = conn
        .send_and_block(&Message::save_file(opts))
        .inspect_err(|err| log::error!("SaveFile failed: {err}"))
        .ok()??;

    let mut iter = MessageIter::from_msg(&reply);

    let Some(got_handle_path) = iter.get_object_path() else {
        log::error!("Response object path is missing");
        return None;
    };

    if handle_path != got_handle_path {
        log::debug!("Detected ancient version of xdg portal, attempting fallback");
        register_response_listener(&mut conn, &got_handle_path);
    }

    wait_for_response(&mut conn, &got_handle_path)
}

fn generate_response_path(conn: &mut Connection, handle_token: &HandleToken) -> CString {
    let unique_name = conn.get_unique_name();
    let unique_name = unique_name.to_str().unwrap();
    let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
    let handle_token = handle_token.0.to_str().unwrap();
    CString::new(format!(
        "/org/freedesktop/portal/desktop/request/{unique_identifier}/{handle_token}"
    ))
    .unwrap()
}

fn register_response_listener(conn: &mut Connection, handle_path: &CStr) {
    conn.add_match(
        &CString::new(
            [
                "type='signal'",
                "sender='org.freedesktop.portal.Desktop'",
                &format!("path='{}'", handle_path.to_str().unwrap()),
                "interface='org.freedesktop.portal.Request'",
                "member='Response'",
            ]
            .join(","),
        )
        .unwrap(),
    )
    .inspect_err(|err| log::error!("Failed to add match rule: {err}"))
    .ok();

    conn.flush();
}

#[derive(Debug)]
enum ResponseCode {
    Success = 0,
    // Cancelled = 1,
    // Other = 2,
}

fn wait_for_response(conn: &mut Connection, handle_path: &CStr) -> Option<Response> {
    let mut connected = true;
    loop {
        // Drain before blocking
        while let Some(signal) = conn.pop_message() {
            if signal.is_signal(c"org.freedesktop.portal.Request", c"Response") {
                let Some(path) = signal.get_path() else {
                    log::error!("Response signal is missing a path");
                    return None;
                };

                if path == handle_path {
                    return parse_response(&signal);
                }
            }
        }

        if !connected {
            log::error!("Lost connection to the portal while waiting for a response");
            return None;
        }

        connected = conn.read_write(-1);
    }
}

#[derive(Debug)]
pub enum Response {
    /// The selected paths, and the label of the filter that was selected when the dialog was closed
    Selected(Vec<PathBuf>, Option<CString>),
    Canceled,
}

fn parse_response(msg: &Message) -> Option<Response> {
    let mut iter = MessageIter::from_msg(msg);

    let Some(response_code) = iter.get_u32() else {
        log::error!("Response code missing");
        return None;
    };
    if response_code != ResponseCode::Success as u32 {
        return Some(Response::Canceled);
    }

    if !iter.next() {
        log::error!("Body of the response is empty");
        return None;
    }

    if iter.get_arg_type() != ffi::DBUS_TYPE_ARRAY {
        log::error!("Body of the response is not an array");
        return None;
    }

    let mut dict_iter = iter.iter_recurse();
    let mut uris = None;
    let mut current_filter = None;

    while dict_iter.get_arg_type() == ffi::DBUS_TYPE_DICT_ENTRY {
        let mut entry_iter = dict_iter.iter_recurse();

        let Some(key) = entry_iter.get_string() else {
            log::error!("Wrong type of a dict key");
            dict_iter.next();
            continue;
        };

        entry_iter.next();
        if key.as_c_str() == c"uris" {
            if entry_iter.get_arg_type() == ffi::DBUS_TYPE_VARIANT {
                let mut var_iter = entry_iter.iter_recurse();
                uris = Some(uris_to_paths(var_iter.get_string_array()));
            } else {
                log::error!(
                    "Response.uris type {} != VARIANT",
                    entry_iter.get_arg_type()
                );
            }
        } else if key.as_c_str() == c"current_filter" {
            current_filter = parse_filter_label(&mut entry_iter);
        }

        dict_iter.next();
    }

    let Some(uris) = uris else {
        log::error!("Response.uris was not found");
        return None;
    };
    Some(Response::Selected(uris, current_filter))
}

/// The label of a `current_filter` entry, a variant holding `(sa(us))`
fn parse_filter_label(entry_iter: &mut MessageIter) -> Option<CString> {
    if entry_iter.get_arg_type() != ffi::DBUS_TYPE_VARIANT {
        log::error!(
            "Response.current_filter type {} != VARIANT",
            entry_iter.get_arg_type()
        );
        return None;
    }
    let mut var_iter = entry_iter.iter_recurse();
    if var_iter.get_arg_type() != ffi::DBUS_TYPE_STRUCT {
        log::error!(
            "Response.current_filter type {} != STRUCT",
            var_iter.get_arg_type()
        );
        return None;
    }
    var_iter.iter_recurse().get_string()
}
