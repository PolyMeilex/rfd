use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

use serde::Serialize;
use zbus::zvariant::Type;

/// A file name represented as a nul-terminated byte array.
#[derive(Type, Debug, Default, PartialEq)]
#[zvariant(signature = "ay")]
pub struct FilePath(CString);

impl FilePath {
    pub fn new<T: AsRef<Path>>(s: T) -> Result<Self, super::Error> {
        let c_string = CString::new(s.as_ref().as_os_str().as_bytes())
            .map_err(|err| super::Error::NulTerminated(err.nul_position()))?;

        Ok(Self(c_string))
    }
}

impl Serialize for FilePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.as_bytes_with_nul())
    }
}
