use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{DeserializeDict, OwnedObjectPath, SerializeDict, Type};

use crate::backend::xdg_impl::desktop::request::Request;

use super::{
    super::{Error, FilePath, WindowIdentifier},
    HandleToken,
};

/// A file filter, to limit the available file choices to a mimetype or a glob
/// pattern.
#[derive(Clone, Serialize, Deserialize, Type, Debug, PartialEq)]
pub struct FileFilter(String, Vec<(FilterType, String)>);

#[derive(Clone, Serialize_repr, Deserialize_repr, Debug, Type, PartialEq)]
#[repr(u32)]
enum FilterType {
    GlobPattern = 0,
    MimeType = 1,
}

impl FileFilter {
    /// Create a new file filter
    ///
    /// # Arguments
    ///
    /// * `label` - user-visible name of the file filter.
    pub fn new(label: &str) -> Self {
        Self(label.to_owned(), vec![])
    }

    /// Adds a glob pattern to the file filter.
    pub fn glob(mut self, pattern: &str) -> Self {
        self.1.push((FilterType::GlobPattern, pattern.to_owned()));
        self
    }
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct OpenFileOptions {
    pub handle_token: HandleToken,
    pub accept_label: Option<String>,
    pub modal: Option<bool>,
    pub multiple: Option<bool>,
    pub directory: Option<bool>,
    pub filters: Vec<FileFilter>,
    pub current_filter: Option<FileFilter>,
    pub current_folder: Option<FilePath>,
}

#[derive(SerializeDict, Type, Debug, Default)]
#[zvariant(signature = "dict")]
pub struct SaveFileOptions {
    pub handle_token: HandleToken,
    pub accept_label: Option<String>,
    pub modal: Option<bool>,
    pub current_name: Option<String>,
    pub current_folder: Option<FilePath>,
    pub current_file: Option<FilePath>,
    pub filters: Vec<FileFilter>,
    pub current_filter: Option<FileFilter>,
}

/// A response of [`OpenFileRequest`], [`SaveFileRequest`] or
/// [`SaveFilesRequest`].
#[derive(Default, Debug, Type, DeserializeDict)]
#[zvariant(signature = "dict")]
pub struct SelectedFiles {
    uris: Vec<url::Url>,
}

impl SelectedFiles {
    /// The selected files uris.
    pub fn uris(&self) -> &[url::Url] {
        self.uris.as_slice()
    }
}

#[derive(Debug, Default)]
pub struct OpenFileRequest;

impl OpenFileRequest {
    pub fn send(
        identifier: Option<WindowIdentifier>,
        title: &str,
        options: &OpenFileOptions,
    ) -> Result<Request, Error> {
        Ok(Request(open_file(identifier.as_ref(), title, options)?))
    }
}

#[derive(Debug, Default)]
pub struct SaveFileRequest;

impl SaveFileRequest {
    /// Send the request.
    pub fn send(
        identifier: Option<WindowIdentifier>,
        title: &str,
        options: &SaveFileOptions,
    ) -> Result<Request, Error> {
        Ok(Request(save_file(identifier.as_ref(), title, options)?))
    }
}

use zbus::blocking::{proxy::SignalIterator, Connection, Proxy};

use crate::backend::xdg_impl::desktop::request::Response;

const DESKTOP_DESTINATION: &str = "org.freedesktop.portal.Desktop";
const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";

fn listen_for_response(
    connection: &Connection,
    handle_token: &HandleToken,
) -> (OwnedObjectPath, SignalIterator<'static>) {
    let unique_name = connection.unique_name().unwrap();
    let unique_identifier = unique_name.trim_start_matches(':').replace('.', "_");
    let path = OwnedObjectPath::try_from(format!(
        "{DESKTOP_PATH}/request/{unique_identifier}/{handle_token}"
    ))
    .unwrap();

    (
        path.clone(),
        Proxy::new(
            connection,
            DESKTOP_DESTINATION,
            path,
            "org.freedesktop.portal.Request",
        )
        .unwrap()
        .receive_signal("Response")
        .unwrap(),
    )
}

fn call_method(
    connection: &Connection,
    method: &str,
    body: impl serde::ser::Serialize + zbus::zvariant::DynamicType,
) -> zbus::Result<zbus::Message> {
    connection.call_method(
        Some(DESKTOP_DESTINATION),
        DESKTOP_PATH,
        Some("org.freedesktop.portal.FileChooser"),
        method,
        &body,
    )
}

fn to_string_or_empty(id: Option<&WindowIdentifier>) -> std::borrow::Cow<'static, str> {
    match id {
        Some(id) => std::borrow::Cow::Owned(id.to_string()),
        None => std::borrow::Cow::Borrowed(""), // No allocation
    }
}

fn open_file(
    identifier: Option<&WindowIdentifier>,
    title: &str,
    options: &OpenFileOptions,
) -> zbus::Result<Response> {
    let connection = Connection::session().unwrap();

    let (res_path, mut response) = listen_for_response(&connection, &options.handle_token);

    let identifier = to_string_or_empty(identifier);
    let method_call_result =
        call_method(&connection, "OpenFile", (identifier, title, &options)).unwrap();

    let obj_path: OwnedObjectPath = method_call_result.body().deserialize().unwrap();

    if obj_path != res_path {
        return Ok(Response::err());
    }

    let res = response.next().unwrap();

    let res: Response = res.body().deserialize().unwrap();

    Ok(res)
}

fn save_file(
    identifier: Option<&WindowIdentifier>,
    title: &str,
    options: &SaveFileOptions,
) -> zbus::Result<Response> {
    let connection = Connection::session().unwrap();

    let (res_path, mut response) = listen_for_response(&connection, &options.handle_token);

    let identifier = to_string_or_empty(identifier);
    let method_call_result =
        call_method(&connection, "SaveFile", (identifier, title, &options)).unwrap();

    let obj_path: OwnedObjectPath = method_call_result.body().deserialize().unwrap();

    if obj_path != res_path {
        return Ok(Response::err());
    }

    let res = response.next().unwrap();

    let res: Response = res.body().deserialize().unwrap();

    Ok(res)
}
