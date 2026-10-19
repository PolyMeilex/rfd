use super::utils::GtkGlobalThread;

use glib::prelude::*;

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

use super::AsGtkDialog;

struct FutureState<R, D> {
    waker: Option<Waker>,
    data: Option<R>,
    dialog: Option<D>,
}

unsafe impl<R, D> Send for FutureState<R, D> {}

pub(super) struct GtkDialogFuture<R, D> {
    state: Arc<Mutex<FutureState<R, D>>>,
}

unsafe impl<R, D> Send for GtkDialogFuture<R, D> {}

impl<R: Default + 'static, D: AsGtkDialog + 'static> GtkDialogFuture<R, D> {
    pub fn new<B, F>(build: B, cb: F) -> Self
    where
        B: FnOnce() -> D + Send + 'static,
        F: Fn(&mut D, i32) -> R + Send + 'static,
    {
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
            dialog: None,
        }));

        {
            let state = state.clone();
            let callback = {
                let state = state.clone();

                move |res_id| {
                    let mut state = state.lock().unwrap();

                    if let Some(mut dialog) = state.dialog.take() {
                        state.data = Some(cb(&mut dialog, res_id));
                    }

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                }
            };

            GtkGlobalThread::instance().run(move || {
                let mut state = state.lock().unwrap();
                state.dialog = Some(build());

                let dialog = state.dialog.as_ref().unwrap();
                unsafe { dialog.show() };

                dialog
                    .object()
                    .connect_local("response", false, move |values| {
                        let res_id = values[1]
                            .get::<i32>()
                            .expect("response signal should carry an i32 response id");
                        callback(res_id);
                        None
                    });
            });
        }

        Self { state }
    }
}

impl<R, D> std::future::Future for GtkDialogFuture<R, D> {
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
