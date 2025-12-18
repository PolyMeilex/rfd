use std::{
    fmt,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receiver<T>(Arc<Channel<T>>);
impl<T> Unpin for Receiver<T> {}

pub struct Sender<T>(Arc<Channel<T>>);
impl<T> Unpin for Sender<T> {}

#[derive(Default)]
enum State<T> {
    Ready(T),
    #[default]
    Pending,
    Canceled,
}

struct Inner<T> {
    rx_waker: Option<Waker>,
    state: State<T>,
}

struct Channel<T>(Mutex<Inner<T>>);

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Channel::new());
    (Sender(inner.clone()), Receiver(inner))
}

impl<T> Channel<T> {
    fn new() -> Self {
        Self(Mutex::new(Inner {
            rx_waker: None,
            state: State::Pending,
        }))
    }

    fn send(&self, t: T) -> Result<(), T> {
        let Ok(mut inner) = self.0.lock() else {
            debug_assert!(false, "Lock poisoned");
            return Err(t);
        };

        inner.state = State::Ready(t);

        // Wake is called after this in `Self::drop_tx`
        Ok(())
    }

    fn drop_tx(&self) {
        let Ok(mut inner) = self.0.lock() else {
            debug_assert!(false, "Lock poisoned");
            return;
        };

        match inner.state {
            State::Ready(_) => {}
            State::Pending | State::Canceled => {
                inner.state = State::Canceled;
            }
        }

        if let Some(waker) = inner.rx_waker.take() {
            drop(inner);
            waker.wake();
        }
    }

    fn recv(&self, cx: &mut Context<'_>) -> Poll<Result<T, Canceled>> {
        let Ok(mut inner) = self.0.lock() else {
            debug_assert!(false, "Lock poisoned");
            return Poll::Ready(Err(Canceled));
        };

        match std::mem::take(&mut inner.state) {
            State::Ready(v) => Poll::Ready(Ok(v)),
            State::Canceled => {
                inner.state = State::Canceled;
                Poll::Ready(Err(Canceled))
            }
            State::Pending => {
                inner.state = State::Pending;
                inner.rx_waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<T> Sender<T> {
    pub fn send(self, t: T) -> Result<(), T> {
        let res = self.0.send(t);
        drop(self);
        res
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.0.drop_tx()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

impl fmt::Display for Canceled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oneshot canceled")
    }
}

impl std::error::Error for Canceled {}

impl<T> Future for Receiver<T> {
    type Output = Result<T, Canceled>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<T, Canceled>> {
        self.0.recv(cx)
    }
}
