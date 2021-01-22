use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

use super::super::utils::init_com;
use super::dialog_ffi::{IDialog, OutputFrom};

struct FutureState<R> {
    waker: Option<Waker>,
    data: Option<R>,
}

unsafe impl<R> Send for FutureState<R> {}

pub struct AsyncDialog<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

impl<R: OutputFrom<IDialog> + Send + 'static> AsyncDialog<R> {
    pub(crate) fn new<F: FnOnce() -> Option<IDialog> + Send + Sync + 'static>(init: F) -> Self {
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
        }));

        {
            let state = state.clone();
            std::thread::spawn(move || {
                let err = init_com(|| {
                    if let Some(dialog) = init() {
                        let ok = dialog.show().is_ok();

                        let mut state = state.lock().unwrap();
                        if ok {
                            state.data = Some(OutputFrom::from(&dialog));
                        } else {
                            state.data = Some(OutputFrom::get_failed());
                        }
                    } else {
                        state.lock().unwrap().data = Some(OutputFrom::get_failed());
                    }
                });

                let mut state = state.lock().unwrap();

                if err.is_err() {
                    state.data = Some(OutputFrom::get_failed());
                }

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            });
        }

        Self { state }
    }
}

impl<R> Into<DialogFuture<R>> for AsyncDialog<R> {
    fn into(self) -> DialogFuture<R> {
        DialogFuture { state: self.state }
    }
}

pub struct DialogFuture<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

unsafe impl<R> Send for DialogFuture<R> {}

impl<R> std::future::Future for DialogFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        if state.data.is_some() {
            Poll::Ready(state.data.take().unwrap())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
