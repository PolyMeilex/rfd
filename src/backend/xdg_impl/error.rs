use super::desktop::request::ResponseError;

#[derive(Debug)]
#[non_exhaustive]
/// The error type for ashpd.
pub enum Error {
    /// The portal request didn't succeed.
    Response(ResponseError),
    /// A zbus::fdo specific error.
    Zbus(zbus::Error),
    /// A signal returned no response.
    NoResponse,
    /// An error indicating that an interior nul byte was found
    NulTerminated(usize),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Response(e) => write!(f, "Portal request didn't succeed: {e}"),
            Self::Zbus(e) => write!(f, "ZBus Error: {e}"),
            Self::NoResponse => f.write_str("Portal error: no response"),
            Self::NulTerminated(u) => write!(f, "Nul byte found in provided data at position {u}"),
        }
    }
}

impl From<ResponseError> for Error {
    fn from(e: ResponseError) -> Self {
        Self::Response(e)
    }
}

impl From<zbus::Error> for Error {
    fn from(e: zbus::Error) -> Self {
        Self::Zbus(e)
    }
}
