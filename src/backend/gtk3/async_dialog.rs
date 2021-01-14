use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use std::task::{Context, Poll, Waker};

use super::{GtkDialog, OutputFrom};

use lazy_static::lazy_static;

type Runner = dyn FnOnce() + Send + Sync + 'static;

pub struct GtkThread {
    tx: Sender<Box<Runner>>,
}

unsafe impl Send for GtkThread {}
unsafe impl Sync for GtkThread {}

impl GtkThread {
    fn new() -> Self {
        let (tx, rx) = channel::<Box<Runner>>();

        std::thread::spawn(move || {
            for cb in rx {
                cb();
            }
        });

        Self { tx }
    }

    fn push<F: FnOnce() + Send + Sync + 'static>(&self, cb: F) {
        self.tx.send(Box::new(cb)).unwrap();
    }
}

lazy_static! {
    pub static ref GTK_THREAD: GtkThread = GtkThread::new();
}

struct FutureState<R> {
    waker: Option<Waker>,
    data: Option<R>,
}

unsafe impl<R> Send for FutureState<R> {}
unsafe impl Send for GtkDialog {}
unsafe impl Sync for GtkDialog {}

pub struct AsyncDialog<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

impl<R: OutputFrom<GtkDialog> + Send + 'static> AsyncDialog<R> {
    pub(crate) fn new<F: FnOnce() -> GtkDialog + Send + Sync + 'static>(init: F) -> Self {
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            data: None,
        }));

        {
            let state = state.clone();

            let cb = move || {
                let mut state = state.lock().unwrap();

                let dialog = if super::gtk_init_check() {
                    Some(init())
                } else {
                    None
                };

                if let Some(dialog) = dialog {
                    let res_id = dialog.run();

                    state.data = Some(OutputFrom::from(&dialog, res_id));
                    std::mem::drop(dialog);
                } else {
                    state.data = Some(OutputFrom::get_failed());
                }

                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            };
            GTK_THREAD.push(cb);
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

use gobject_sys::GCallback;
use gtk_sys::{GtkFileChooser, GtkResponseType};
use std::ffi::c_void;
use std::ptr;
use std::{ffi::CStr, os::raw::c_char};

unsafe fn connect_raw<F>(
    receiver: *mut gobject_sys::GObject,
    signal_name: *const c_char,
    trampoline: GCallback,
    closure: *mut F,
) {
    use std::mem;

    use glib_sys::gpointer;

    unsafe extern "C" fn destroy_closure<F>(ptr: *mut c_void, _: *mut gobject_sys::GClosure) {
        // destroy
        Box::<F>::from_raw(ptr as *mut _);
    }
    assert_eq!(mem::size_of::<*mut F>(), mem::size_of::<gpointer>());
    assert!(trampoline.is_some());
    let handle = gobject_sys::g_signal_connect_data(
        receiver,
        signal_name,
        trampoline,
        closure as *mut _,
        Some(destroy_closure::<F>),
        0,
    );
    assert!(handle > 0);
    // from_glib(handle)
}

pub unsafe fn connect_response<F: Fn(GtkResponseType) + 'static>(
    dialog: *mut GtkFileChooser,
    f: F,
) {
    use std::mem::transmute;

    unsafe extern "C" fn response_trampoline<F: Fn(GtkResponseType) + 'static>(
        this: *mut gtk_sys::GtkDialog,
        res: GtkResponseType,
        f: glib_sys::gpointer,
    ) {
        let f: &F = &*(f as *const F);

        f(res);
        // f(
        //     &Dialog::from_glib_borrow(this).unsafe_cast_ref(),
        //     from_glib(response_id),
        // )
    }
    let f: Box<F> = Box::new(f);
    connect_raw(
        dialog as *mut _,
        b"response\0".as_ptr() as *const _,
        Some(transmute::<_, unsafe extern "C" fn()>(
            response_trampoline::<F> as *const (),
        )),
        Box::into_raw(f),
    );
}
