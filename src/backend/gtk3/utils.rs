use lazy_static::lazy_static;

use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Ensures that gtk is allways called from one thread at the time
pub struct GtkGlobalMutex {
    locker: Mutex<()>,
}

unsafe impl Send for GtkGlobalMutex {}
unsafe impl Sync for GtkGlobalMutex {}

impl GtkGlobalMutex {
    fn new() -> Self {
        Self {
            locker: Mutex::new(()),
        }
    }

    pub(super) fn run_locked<T, F: FnOnce() -> T>(&self, cb: F) -> T {
        let _guard = self.locker.lock().unwrap();
        cb()
    }
}

lazy_static! {
    pub static ref GTK_MUTEX: GtkGlobalMutex = GtkGlobalMutex::new();
}

/// # Event Hadnler
/// Counts amout of iteration requests
/// When amount of requests goes above 0 it spawns GtkThread and starts iteration
/// When amount of requests reqches 0 it stops GtkThread, and goes idle
pub struct GtkEventHandler {
    thread: Mutex<Option<GtkThread>>,
    request_count: Arc<AtomicUsize>,
}

unsafe impl Send for GtkEventHandler {}
unsafe impl Sync for GtkEventHandler {}

lazy_static! {
    pub static ref GTK_EVENT_HANDLER: GtkEventHandler = GtkEventHandler::new();
}

impl GtkEventHandler {
    fn new() -> Self {
        let thread = Mutex::new(None);
        let request_count = Arc::new(AtomicUsize::new(0));
        Self {
            thread,
            request_count,
        }
    }

    /// Ask GtkEventHandler to start event iteration
    /// When iteration is no longer needed, just drop IterationRequest.
    /// And when numer of requests reaches 0 iteration will be stoped
    pub fn request_iteration_start(&self) -> IterationRequest {
        let mut thread = self.thread.lock().unwrap();
        if thread.is_none() {
            thread.replace(GtkThread::new());
        }

        IterationRequest::new(self.request_count.clone())
    }

    fn iteration_stop(&self) {
        self.thread.lock().unwrap().take();
    }

    fn request_iteration_stop(&self) {
        if self.request_count.load(Ordering::Acquire) == 0 {
            self.iteration_stop();
        }
    }
}

pub struct IterationRequest {
    request_count: Arc<AtomicUsize>,
}

impl IterationRequest {
    fn new(request_count: Arc<AtomicUsize>) -> Self {
        request_count.fetch_add(1, Ordering::Relaxed);
        Self { request_count }
    }
}

impl Drop for IterationRequest {
    fn drop(&mut self) {
        self.request_count.fetch_sub(1, Ordering::Release);
        GTK_EVENT_HANDLER.request_iteration_stop();
    }
}

/// Thread that iterates gtk events
struct GtkThread {
    _handle: JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl GtkThread {
    fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));

        let _handle = {
            let running = running.clone();
            std::thread::spawn(move || {
                while running.load(Ordering::Acquire) {
                    GTK_MUTEX.run_locked(|| unsafe {
                        while gtk_sys::gtk_events_pending() == 1 {
                            gtk_sys::gtk_main_iteration();
                        }
                    });
                }
            })
        };

        Self { _handle, running }
    }
}

impl Drop for GtkThread {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
    }
}

pub fn gtk_init_check() -> bool {
    unsafe { gtk_sys::gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 1 }
}

/// gtk_main_iteration()
pub unsafe fn wait_for_cleanup() {
    while gtk_sys::gtk_events_pending() == 1 {
        gtk_sys::gtk_main_iteration();
    }
}
