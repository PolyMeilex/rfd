//! Native event processing for standalone macOS applications.
//!
//! Use [`run_with_native_events`](crate::macos::run_with_native_events) when
//! no GUI framework owns the application's native event loop.

use std::{
    error::Error,
    fmt, io,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use block2::RcBlock;
use dispatch2::{DispatchQueue, MainThreadBound};
use objc2::{
    rc::{autoreleasepool, Retained},
    MainThreadMarker,
};
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType};
use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopMode};
use objc2_foundation::NSPoint;

use super::utils::{activate_cocoa_multithreading, PolicyManager};

/// A failure to start rfd-owned native event processing.
#[derive(Debug)]
pub enum NativeLoopError {
    /// The helper was called outside the OS main thread.
    NotMainThread,
    /// An AppKit application loop is already running.
    ApplicationAlreadyRunning,
    /// An AppKit modal session is already running.
    ModalSessionActive,
    /// AppKit could not create the event used to wake its loop during shutdown.
    ShutdownEventUnavailable,
    /// Core Foundation did not provide the main run loop.
    MainRunLoopUnavailable,
    /// Core Foundation did not provide the default run-loop mode.
    DefaultRunLoopModeUnavailable,
    /// The application worker could not be created.
    WorkerStart(io::Error),
    /// The startup channel closed or reported that AppKit was not running.
    StartupConfirmationFailed,
}

impl fmt::Display for NativeLoopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NotMainThread => "native event ownership requires the OS main thread",
            Self::ApplicationAlreadyRunning => "a native application loop is already running",
            Self::ModalSessionActive => "a native modal session is already running",
            Self::ShutdownEventUnavailable => "cannot create the native shutdown wake event",
            Self::MainRunLoopUnavailable => "the main native run loop is unavailable",
            Self::DefaultRunLoopModeUnavailable => {
                "the default native run-loop mode is unavailable"
            }
            Self::WorkerStart(error) => {
                return write!(formatter, "cannot start the application worker: {error}");
            }
            Self::StartupConfirmationFailed => "the native event loop did not confirm startup",
        };
        formatter.write_str(message)
    }
}

impl Error for NativeLoopError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::WorkerStart(error) => Some(error),
            _ => None,
        }
    }
}

/// Run native events on the main thread and application code on a worker.
///
/// Supports both `FileDialog` and `AsyncFileDialog`.
/// Without a suitable parent window, async dialogs block until closed.
/// This function does not run async futures.
///
/// # Usage
///
/// Call once from the OS main thread, without another framework owning or
/// preparing to own its event loop. Runtime checks cannot detect every framework's
/// ownership, especially before its loop starts; the caller must choose this mode.
///
/// Keep the closure running while the application needs native dialogs,
/// including waits between requests. Finish all dialog calls before returning.
///
/// # Example
///
/// Console input waits on the worker while main continues processing native events.
///
/// ```no_run
/// use std::{error::Error, io};
/// use rfd::{macos::run_with_native_events, FileDialog};
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let work_result = run_with_native_events(|| -> io::Result<()> {
///         let mut command = String::new();
///         loop {
///             println!("Press Enter to select a file, or enter q to quit.");
///             command.clear();
///             if io::stdin().read_line(&mut command)? == 0 || command.trim() == "q" {
///                 return Ok(());
///             }
///             println!("Selection: {:?}", FileDialog::new().pick_file());
///         }
///     })?;
///     work_result?;
///     Ok(())
/// }
/// ```
///
/// # Lifecycle
///
/// 1. Main starts processing native events; the worker waits for startup
///    confirmation without a timeout.
/// 2. Application code runs on the worker while main continues processing events,
///    including deferred dialog dismissal.
/// 3. When application code finishes, the worker requests shutdown. After the native
///    loop returns, the caller joins the worker.
///
/// The closure's value is returned inside `Ok`, including when that value is
/// itself a `Result`. Returning does not guarantee that all dismissal animation
/// has finished.
///
/// # Errors
///
/// Returns [`NativeLoopError`] if called off main, if an AppKit loop or modal
/// session is already running, if a required resource or worker cannot be
/// created, or if startup confirmation fails.
/// The application closure is not called on error. AppKit may already have been
/// initialized; returning an error does not undo that initialization.
///
/// # Panics
///
/// A panic from the application closure is resumed on the caller after the
/// native loop stops and the worker joins.
pub fn run_with_native_events<F, T>(work: F) -> Result<T, NativeLoopError>
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    let main = MainThreadMarker::new().ok_or(NativeLoopError::NotMainThread)?;
    autoreleasepool(|_| {
        let app = NSApplication::sharedApplication(main);
        if app.isRunning() {
            return Err(NativeLoopError::ApplicationAlreadyRunning);
        }
        if app.modalWindow().is_some() {
            return Err(NativeLoopError::ModalSessionActive);
        }
        // Keep the final owner on main through worker joining. Releasing these
        // resources on a worker could otherwise wait on an already-stopped loop.
        let shutdown = Arc::new(NativeLoopShutdown::new(main)?);
        // Restore the previous activation policy when this scope ends.
        let _activation_policy_guard = PolicyManager::new(main);
        activate_cocoa_multithreading();
        app.finishLaunching();

        thread::scope(|scope| {
            // Wait for the main loop to service a callback before allowing
            // application work to request dialogs.
            let (ready, started) = mpsc::channel::<bool>();
            let worker_shutdown = Arc::clone(&shutdown);
            let worker = thread::Builder::new()
                .spawn_scoped(scope, move || {
                    // Request native-loop shutdown even when application work panics.
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        if !matches!(started.recv(), Ok(true)) {
                            return Err(NativeLoopError::StartupConfirmationFailed);
                        }
                        Ok(work())
                    }));
                    worker_shutdown.request_stop();
                    result
                })
                .map_err(NativeLoopError::WorkerStart)?;
            DispatchQueue::main().exec_async(move || {
                // SAFETY: DispatchQueue::main executes on the OS main thread.
                let main = unsafe { MainThreadMarker::new_unchecked() };
                let app = NSApplication::sharedApplication(main);
                let _ = ready.send(app.isRunning());
            });

            // If another caller stops AppKit first, resume it until our shutdown
            // callback runs. Joining sooner could strand a worker waiting on main.
            loop {
                app.run();
                if shutdown.stop_callback_ran.load(Ordering::Acquire) {
                    break;
                }
            }
            match worker.join() {
                Ok(Ok(result)) => result,
                Ok(Err(panic)) | Err(panic) => resume_unwind(panic),
            }
        })
    })
}

/// Owns the resources used to stop the AppKit loop.
struct NativeLoopShutdown {
    /// Event that lets AppKit's event wait return and observe `stop()`.
    shutdown_wake_event: MainThreadBound<Retained<NSEvent>>,
    /// Main run loop that executes the shutdown callback.
    run_loop: MainThreadBound<CFRetained<CFRunLoop>>,
    /// Mode used to schedule the shutdown callback.
    default_mode: MainThreadBound<&'static CFRunLoopMode>,
    /// Set after the callback calls `stop()` and posts the event.
    /// This does not mean the native loop has returned.
    stop_callback_ran: AtomicBool,
}

impl NativeLoopShutdown {
    /// Prepare all shutdown resources on main before starting the worker.
    fn new(main: MainThreadMarker) -> Result<Self, NativeLoopError> {
        let shutdown_wake_event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
            NSEventType::ApplicationDefined, NSPoint::ZERO, NSEventModifierFlags::empty(),
            0.0, 0, None, 0, 0, 0,
        ).ok_or(NativeLoopError::ShutdownEventUnavailable)?;
        let run_loop = CFRunLoop::main().ok_or(NativeLoopError::MainRunLoopUnavailable)?;
        // SAFETY: Core Foundation publishes this process-lifetime mode constant.
        let default_mode = unsafe { kCFRunLoopDefaultMode }
            .ok_or(NativeLoopError::DefaultRunLoopModeUnavailable)?;

        Ok(Self {
            shutdown_wake_event: MainThreadBound::new(shutdown_wake_event, main),
            run_loop: MainThreadBound::new(run_loop, main),
            default_mode: MainThreadBound::new(default_mode, main),
            stop_callback_ran: AtomicBool::new(false),
        })
    }

    /// Request shutdown without blocking the worker.
    ///
    /// 1. Dispatch the request to main.
    /// 2. Schedule the stop callback in the default run-loop mode and wake the loop.
    /// 3. When that callback runs, stop AppKit and post its wake event.
    ///
    /// Using the default mode defers stopping outside AppKit's modal and tracking modes.
    fn request_stop(self: Arc<Self>) {
        DispatchQueue::main().exec_async(move || {
            // SAFETY: this callback is dispatched to the main queue.
            let main = unsafe { MainThreadMarker::new_unchecked() };
            let shutdown = Arc::clone(&self);
            let stop = RcBlock::new(move || {
                // SAFETY: this block is scheduled exclusively on the main run loop.
                let main = unsafe { MainThreadMarker::new_unchecked() };
                let app = NSApplication::sharedApplication(main);
                app.stop(None);
                // AppKit checks stop() after an NSEvent; waking CFRunLoop alone
                // does not make its event wait return to that check.
                app.postEvent_atStart(shutdown.shutdown_wake_event.get(main), true);
                shutdown.stop_callback_ran.store(true, Ordering::Release);
            });
            let run_loop = self.run_loop.get(main);
            let default_mode = self.default_mode.get(main);
            // SAFETY: default_mode is a process-lifetime CFString. Core Foundation
            // copies the block, which retains the resources used by the callback.
            unsafe {
                run_loop.perform_block(Some((*default_mode).as_ref()), Some(&stop));
            }
            run_loop.wake_up();
        });
    }
}
