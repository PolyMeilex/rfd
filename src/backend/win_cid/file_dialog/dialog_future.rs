use super::super::thread_future::ThreadFuture;
use super::super::utils::init_com;
use super::dialog_ffi::IDialog;

use windows::core::Result;

use crate::file_handle::FileHandle;

pub fn single_return_future<F: FnOnce() -> Result<IDialog> + Send + 'static>(
    build: F,
) -> ThreadFuture<Option<FileHandle>> {
    ThreadFuture::new(move |data| {
        let ret: Result<()> = (|| {
            init_com(|| {
                let dialog = build()?;
                dialog.show()?;

                let path = dialog.get_result().ok().map(FileHandle::wrap);
                *data = Some(path);

                Ok(())
            })?
        })();

        if ret.is_err() {
            *data = Some(None);
        }
    })
}

pub fn multiple_return_future<F: FnOnce() -> Result<IDialog> + Send + 'static>(
    build: F,
) -> ThreadFuture<Option<Vec<FileHandle>>> {
    ThreadFuture::new(move |data| {
        let ret: Result<()> = (|| {
            init_com(|| {
                let dialog = build()?;
                dialog.show()?;

                let list = dialog
                    .get_results()
                    .ok()
                    .map(|r| r.into_iter().map(FileHandle::wrap).collect());
                *data = Some(list);

                Ok(())
            })?
        })();

        if ret.is_err() {
            *data = Some(None);
        }
    })
}
