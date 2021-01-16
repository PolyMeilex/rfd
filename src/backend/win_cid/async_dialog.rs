use core::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

struct FutureState<R> {
    waker: Option<Waker>,
    data: Option<R>,
}

unsafe impl<R> Send for FutureState<R> {}

pub struct AsyncDialog<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

// impl<R: OutputFrom<GtkDialog> + Send + 'static> AsyncDialog<R> {
//     pub(crate) fn new<F: FnOnce() -> () + Send + Sync + 'static>(init: F) -> Self {
//         let state = Arc::new(Mutex::new(FutureState {
//             waker: None,
//             data: None,
//         }));

//         Self { state }
//     }
// }

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
