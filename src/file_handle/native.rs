use std::{
    io,
    path::{Path, PathBuf},
};

use super::super::oneshot;

/// FileHandle is a way of abstracting over a file returned by a dialog
#[derive(Clone)]
pub struct FileHandle(PathBuf);

impl FileHandle {
    /// On native platforms it wraps path.
    ///
    /// On `WASM32` it wraps JS `File` object.
    pub(crate) fn wrap(path_buf: PathBuf) -> Self {
        Self(path_buf)
    }

    /// Get name of a file
    pub fn file_name(&self) -> String {
        self.0
            .file_name()
            .and_then(|f| f.to_str())
            .map(|f| f.to_string())
            .unwrap_or_default()
    }

    /// Gets path to a file.
    ///
    /// Does not exist in `WASM32`
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Reads a file asynchronously.
    ///
    /// On native platforms it spawns a `std::thread` in the background.
    ///
    /// `This fn exists solely to keep native api in pair with async only web api.`
    pub async fn read(&self) -> Vec<u8> {
        let (tx, rx) = oneshot::channel();
        let path = self.0.clone();

        std::thread::Builder::new()
            .name("rfd_file_read".into())
            .spawn(move || {
                tx.send(std::fs::read(path)).unwrap();
            })
            .unwrap();

        match rx.await {
            Ok(res) => res,
            Err(_) => Err(io::Error::other("Read tread panicked")),
        }
        // TODO: Move to io::Result
        .unwrap()
    }

    /// Writes a file asynchronously.
    ///
    /// On native platforms it spawns a `std::thread` in the background.
    ///
    /// `This fn exists solely to keep native api in pair with async only web api.`
    pub async fn write(&self, data: &[u8]) -> std::io::Result<()> {
        let (tx, rx) = oneshot::channel();

        let path = self.0.clone();
        let bytes = data.to_owned();

        std::thread::Builder::new()
            .name("rfd_file_write".into())
            .spawn(move || {
                tx.send(std::fs::write(path, bytes)).unwrap();
            })
            .unwrap();

        match rx.await {
            Ok(res) => res,
            Err(_) => Err(io::Error::other("Write tread panicked")),
        }
    }

    /// Unwraps a `FileHandle` and returns inner type.
    ///
    /// It should be used, if user wants to handle file read themselves
    ///
    /// On native platforms returns path.
    ///
    /// On `WASM32` it returns JS `File` object.
    ///
    /// #### Behind a `file-handle-inner` feature flag
    #[cfg(feature = "file-handle-inner")]
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.path())
    }
}

impl From<PathBuf> for FileHandle {
    fn from(path: PathBuf) -> Self {
        Self::wrap(path)
    }
}

impl From<FileHandle> for PathBuf {
    fn from(file_handle: FileHandle) -> Self {
        file_handle.0
    }
}

impl From<&FileHandle> for PathBuf {
    fn from(file_handle: &FileHandle) -> Self {
        PathBuf::from(file_handle.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "unix")]
    #[test]
    fn write_and_read() {
        futures::executor::block_on(async {
            let path = "/tmp/rfd_test_write_read.txt";
            let handle = FileHandle(path.into());

            handle.write(b"Hello world").await.unwrap();
            let bytes = handle.read().await;

            assert_eq!(bytes, b"Hello world");

            std::fs::remove_file(path).unwrap();
        });
    }
}
