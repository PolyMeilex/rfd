use objc::{class, msg_send, sel, sel_impl};

use cocoa_foundation::base::id;
use cocoa_foundation::base::nil;
use objc::runtime::Object;

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

use super::{OutputFrom, Panel};

pub fn activate_cocoa_multithreading() {
    unsafe {
        let thread: id = msg_send![class!(NSThread), new];
        let _: () = msg_send![thread, start];
    }
}

struct FutureState<R> {
    waker: Option<Waker>,
    panel: Panel,
    data: Option<R>,
}

pub struct AsyncDialog<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

impl<R: OutputFrom<Panel>> AsyncDialog<R> {
    pub(crate) fn new(panel: Panel) -> Self {
        activate_cocoa_multithreading();
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            panel,
            data: None,
        }));

        let app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let was_running: bool = unsafe { msg_send![app, isRunning] };

        let completion = {
            let state = state.clone();

            block::ConcreteBlock::new(move |result: i32| {
                let mut state = state.lock().unwrap();

                let panel = &state.panel;
                state.data = Some(OutputFrom::from(panel, result));

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }

                if !was_running {
                    unsafe {
                        let _: () = msg_send![app, stop: nil];
                    }
                }
            })
        };

        unsafe {
            let state = state.lock().unwrap();
            let _: () = msg_send![*state.panel.panel, beginWithCompletionHandler: &completion];

            if !was_running {
                let _: () = msg_send![app, run];
            }

            std::mem::forget(completion);
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
