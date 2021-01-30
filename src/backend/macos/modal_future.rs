use objc::{class, msg_send, sel, sel_impl};

use cocoa_foundation::base::id;
use cocoa_foundation::base::nil;
use objc::runtime::Object;

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

use super::AsModal;

pub fn activate_cocoa_multithreading() {
    unsafe {
        let thread: id = msg_send![class!(NSThread), new];
        let _: () = msg_send![thread, start];
    }
}

struct FutureState<R, D> {
    waker: Option<Waker>,
    data: Option<R>,
    modal: Option<D>,
}

unsafe impl<R, D> Send for FutureState<R, D> {}

pub(super) struct ModalFuture<R, D> {
    state: Arc<Mutex<FutureState<R, D>>>,
}

unsafe impl<R, D> Send for ModalFuture<R, D> {}

impl<R: 'static, D: AsModal> ModalFuture<R, D> {
    pub fn new<F>(modal: D, cb: F) -> Self
    where
        F: Fn(&mut D, i64) -> R + Send + 'static,
    {
        activate_cocoa_multithreading();

        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
            modal: Some(modal),
        }));

        let completion = {
            let state = state.clone();

            block::ConcreteBlock::new(move |result: i64| {
                let mut state = state.lock().unwrap();

                // take() to drop it when it's safe to do so
                state.data = if let Some(mut modal) = state.modal.take() {
                    Some(cb(&mut modal, result))
                } else {
                    None
                };

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            })
        };

        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let is_running: bool = msg_send![app, isRunning];
            let window: id = msg_send![app, keyWindow];

            // if async exec is possible start sheet modal
            // otherwise fallback to sync
            if is_running && !window.is_null() {
                let _: () = msg_send![
                    state.lock().unwrap().modal.as_ref().unwrap().modal_ptr(),
                    beginSheetModalForWindow: window completionHandler: &completion
                ];
                std::mem::forget(completion);
            } else {
                let ret: i64 = msg_send![
                    state.lock().unwrap().modal.as_ref().unwrap().modal_ptr(),
                    runModal
                ];
                completion.call((ret,));
                std::mem::drop(completion);
            }
        }

        Self { state }
    }
}

impl<R, D> std::future::Future for ModalFuture<R, D> {
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
