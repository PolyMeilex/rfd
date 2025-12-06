pub mod desktop;
mod error;
mod window_identifier;

pub use self::window_identifier::WindowIdentifier;
mod file_path;
pub use self::file_path::FilePath;

pub use self::error::Error;
