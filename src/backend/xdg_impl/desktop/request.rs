use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

use crate::backend::xdg_impl::desktop::SelectedFiles;

use super::super::Error;

#[derive(Debug, Type, serde::Deserialize)]
pub struct Response {
    response: ResponseType,
    results: SelectedFiles,
}

impl Response {
    pub fn err() -> Self {
        Self {
            response: ResponseType::Other,
            results: SelectedFiles::default(),
        }
    }
}

/// An error returned a portal request caused by either the user cancelling the
/// request or something else.
#[derive(Debug, Copy, PartialEq, Eq, Hash, Clone)]
pub enum ResponseError {
    /// The user canceled the request.
    Cancelled,
    /// Something else happened.
    Other,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("Cancelled"),
            Self::Other => f.write_str("Other"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Type)]
/// Possible responses.
pub enum ResponseType {
    /// Success, the request is carried out.
    Success = 0,
    /// The user cancelled the interaction.
    Cancelled = 1,
    /// The user interaction was ended in some other way.
    Other = 2,
}

#[doc(hidden)]
impl From<ResponseError> for ResponseType {
    fn from(err: ResponseError) -> Self {
        match err {
            ResponseError::Other => Self::Other,
            ResponseError::Cancelled => Self::Cancelled,
        }
    }
}

pub struct Request(pub Response);
impl Request {
    pub fn response(self) -> Result<SelectedFiles, Error> {
        match self.0.response {
            ResponseType::Success => Ok(self.0.results),
            ResponseType::Cancelled => Err(ResponseError::Cancelled)?,
            ResponseType::Other => Err(ResponseError::Other)?,
        }
    }
}
