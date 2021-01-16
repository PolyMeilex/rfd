use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct ReaderState {
    res: Option<std::io::Result<Vec<u8>>>,
    waker: Option<Waker>,
}

struct Reader {
    state: Arc<Mutex<ReaderState>>,
}

impl Reader {
    fn new(path: &Path) -> Self {
        let state: Arc<Mutex<ReaderState>> = Arc::new(Mutex::new(Default::default()));

        {
            let path = path.to_owned();
            let state = state.clone();
            std::thread::Builder::new()
                .name("rfd_file_read".into())
                .spawn(move || {
                    let res = std::fs::read(path);

                    let mut state = state.lock().unwrap();
                    state.res.replace(res);

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                })
                .unwrap();
        }

        Self { state }
    }
}

impl Future for Reader {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.res.is_some() {
            let res = state.res.take().unwrap();
            Poll::Ready(res.unwrap())
        } else {
            state.waker.replace(ctx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct FileHandle(PathBuf);

impl FileHandle {
    pub fn wrap(path_buf: PathBuf) -> Self {
        Self(path_buf)
    }

    pub fn file_name(&self) -> String {
        self.0
            .file_name()
            .and_then(|f| f.to_str())
            .map(|f| f.to_string())
            .unwrap_or_default()
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// This fn exists souly to keep native api in pair with async only web api.
    pub async fn read(&self) -> Vec<u8> {
        Reader::new(&self.0).await
    }

    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name())
    }
}
