use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
};

use serde::Serialize;
use zbus::{names::OwnedMemberName, zvariant::Type};

pub(crate) mod request;

pub mod file_chooser;

/// A handle token is a DBus Object Path element.
///
/// Specified in the [`Request`](crate::desktop::Request)  or
/// [`Session`](crate::desktop::Session) object path following this format
/// `/org/freedesktop/portal/desktop/request/SENDER/TOKEN` where sender is the
/// caller's unique name and token is the [`HandleToken`].
///
/// A valid object path element must only contain the ASCII characters
/// `[A-Z][a-z][0-9]_`
#[derive(Serialize, Debug, Type)]
pub(crate) struct HandleToken(OwnedMemberName);

impl Display for HandleToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Default for HandleToken {
    fn default() -> Self {
        let mut token = String::with_capacity(16); // "ashpd_" + 10 chars
        token.push_str("ashpd_");
        for _ in 0..10 {
            token.push(fastrand::alphanumeric());
        }
        Self(OwnedMemberName::try_from(token).unwrap())
    }
}
