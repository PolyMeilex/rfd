use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use block2::{Block, RcBlock};
use objc2::rc::Retained;
use objc2::{ClassType, MainThreadMarker, MainThreadOnly, Message};
use objc2_app_kit::{NSApplication, NSModalResponse, NSWindow};

use super::utils::{activate_cocoa_multithreading, run_on_main};

pub(super) trait AsModal {
    fn inner_modal(&self) -> &(impl InnerModal + 'static);
    /// Return the parent explicitly supplied by the caller.
    fn parent(&self) -> Option<&Retained<NSWindow>>;
}

pub(super) trait InnerModal: ClassType + MainThreadOnly {
    fn begin_modal(&self, window: &NSWindow, handler: &Block<dyn Fn(NSModalResponse)>);
    fn run_modal(&self) -> NSModalResponse;
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

impl<R: 'static + Default, D: AsModal + 'static> ModalFuture<R, D> {
    /// Start the dialog on the main thread.
    ///
    /// Use a sheet when the application is running and a parent is available;
    /// otherwise wait for modal dismissal before returning this future.
    pub fn new<F, DBULD: FnOnce(MainThreadMarker) -> D + Send>(build_modal: DBULD, cb: F) -> Self
    where
        F: Fn(&mut D, isize) -> R + Send + 'static,
    {
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
            modal: None,
        }));

        // Shared completion path: store the result and drop the wrapper on main
        // before waking the waiting task.
        let dialog_callback = move |state: &Mutex<FutureState<R, D>>, result: NSModalResponse| {
            let mut state = state.lock().unwrap();
            state.data = if let Some(mut modal) = state.modal.take() {
                Some((cb)(&mut modal, result))
            } else {
                Some(Default::default())
            };
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        };

        let dialog_state = Arc::clone(&state);
        run_on_main(move |main| {
            activate_cocoa_multithreading();
            let app = NSApplication::sharedApplication(main);
            let running = app.isRunning();
            // Look up existing host windows before building the dialog: the new
            // panel must not become its own fallback sheet parent.
            let default_parent = if running {
                app.mainWindow().or_else(|| app.windows().firstObject())
            } else {
                None
            };
            let modal = build_modal(main);
            let parent = if running {
                modal.parent().cloned().or(default_parent)
            } else {
                None
            };
            // Keep the native dialog alive during the call, even if completion
            // drops its wrapper.
            let inner = modal.inner_modal().retain();
            // Make the wrapper available before native completion can run.
            // Release the mutex before presenting the dialog: completion locks it too.
            dialog_state.lock().unwrap().modal = Some(modal);

            if let Some(window) = parent {
                let completion = RcBlock::new(move |result| {
                    dialog_callback(&dialog_state, result);
                });
                inner.begin_modal(&window, &completion);
                return;
            }

            let result = inner.run_modal();
            dialog_callback(&dialog_state, result);
        });

        Self { state }
    }
}

impl<R, D> Future for ModalFuture<R, D> {
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
