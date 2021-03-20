use objc::{msg_send, runtime::Object, sel, sel_impl};

use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

use super::AsModal;

use super::utils::{activate_cocoa_multithreading, is_main_thread, NSApplication};

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

impl<R: 'static + Default, D: AsModal> ModalFuture<R, D> {
    pub fn new<F, DBULD: FnOnce() -> D + Send>(build_modal: DBULD, cb: F) -> Self
    where
        F: Fn(&mut D, i64) -> R + Send + 'static,
    {
        activate_cocoa_multithreading();

        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
            modal: None,
        }));

        let dialog_callback = move |state: Arc<Mutex<FutureState<R, D>>>, result: i64| {
            let mut state = state.lock().unwrap();
            // take() to drop it when it's safe to do so
            state.data = if let Some(mut modal) = state.modal.take() {
                Some((&cb)(&mut modal, result))
            } else {
                Some(Default::default())
            };
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        };

        let app = NSApplication::shared_application();

        // if async exec is possible start sheet modal
        // otherwise fallback to sync
        if app.is_running() && !app.key_window().is_null() {
            let state = state.clone();
            let main_runner = move || {
                let completion = {
                    let state = state.clone();
                    block::ConcreteBlock::new(move |result: i64| {
                        dialog_callback(state.clone(), result);
                    })
                };

                let window: *mut Object = NSApplication::shared_application().key_window();

                let mut modal = build_modal();
                let modal_ptr = modal.modal_ptr();

                state.lock().unwrap().modal = Some(modal);

                let _: () = unsafe {
                    msg_send![
                        modal_ptr,
                        beginSheetModalForWindow: window completionHandler: &completion
                    ]
                };

                std::mem::forget(completion);
            };

            if !is_main_thread() {
                let main = dispatch::Queue::main();
                main.exec_sync(main_runner);
            } else {
                main_runner();
            }
        } else {
            eprintln!("\n Hi! It looks like you are running async dialog in unsuported environment, I will fallback to sync dialog for you. \n");

            if is_main_thread() {
                let mut modal = build_modal();
                let modal_ptr = modal.modal_ptr();

                state.lock().unwrap().modal = Some(modal);

                let ret: i64 = unsafe { msg_send![modal_ptr, runModal] };

                dialog_callback(state.clone(), ret);
            } else {
                panic!("Fallback Sync Dialog Must Be Spawned On Main Thread (Why? If async dialog is unsuported in this env, it also means that spawining dialogs outside of main thread is also inposible");
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
