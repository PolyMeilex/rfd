use crate::FileDialog;
use std::path::Path;
use std::path::PathBuf;

use objc::{class, msg_send, sel, sel_impl};

use cocoa_foundation::base::id;
use cocoa_foundation::base::nil;
use cocoa_foundation::foundation::{NSArray, NSAutoreleasePool, NSString, NSURL};
use objc::runtime::{Object, YES};
pub use objc::runtime::{BOOL, NO};

pub fn pick_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::open_panel();

    if !opt.filters.is_empty() {
        panel.add_filters(&opt);
    }

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    panel.set_can_choose_directories(NO);
    panel.set_can_choose_files(YES);

    let res = if panel.run_modal() == 1 {
        Some(panel.get_result())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

pub fn save_file<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::save_panel();

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    let res = if panel.run_modal() == 1 {
        Some(panel.get_result())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

pub fn pick_folder<'a>(opt: &FileDialog<'a>) -> Option<PathBuf> {
    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::open_panel();

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    panel.set_can_choose_directories(YES);
    panel.set_can_choose_files(NO);

    let res = if panel.run_modal() == 1 {
        Some(panel.get_result())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

pub fn pick_files<'a>(opt: &FileDialog<'a>) -> Option<Vec<PathBuf>> {
    let pool = unsafe { NSAutoreleasePool::new(nil) };
    let panel = Panel::open_panel();

    if !opt.filters.is_empty() {
        panel.add_filters(&opt);
    }

    if let Some(path) = &opt.starting_directory {
        panel.set_path(path);
    }

    panel.set_can_choose_directories(NO);
    panel.set_can_choose_files(YES);
    panel.set_allows_multiple_selection(YES);

    let res = if panel.run_modal() == 1 {
        Some(panel.get_results())
    } else {
        None
    };

    unsafe { pool.drain() };

    res
}

use objc::rc::StrongPtr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::task::{Context, Poll, Waker};

pub fn activate_cocoa_multithreading() {
    unsafe {
        let thread: id = msg_send![class!(NSThread), new];
        let _: () = msg_send![thread, start];
    }
}

struct FutureState<R> {
    waker: Option<Waker>,
    panel: Panel,
    done: bool,
    data: Option<R>,
}

struct AsyncDialog<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

impl<R: OutputFrom<Panel>> AsyncDialog<R> {
    fn new(panel: Panel) -> Self {
        activate_cocoa_multithreading();
        let state = Arc::new(Mutex::new(FutureState {
            waker: None,
            panel,
            done: false,
            data: None,
        }));

        let app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let was_running: bool = unsafe { msg_send![app, isRunning] };

        let completion = {
            let state = state.clone();

            block::ConcreteBlock::new(move |result: i32| {
                println!("Done");

                let mut state = state.lock().unwrap();

                state.done = true;

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

impl<R> Into<RetFuture<R>> for AsyncDialog<R> {
    fn into(self) -> RetFuture<R> {
        RetFuture { state: self.state }
    }
}

pub struct RetFuture<R> {
    state: Arc<Mutex<FutureState<R>>>,
}

unsafe impl<R> Send for RetFuture<R> {}

impl<R> std::future::Future for RetFuture<R> {
    type Output = R;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        if state.done {
            Poll::Ready(state.data.take().unwrap())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn async_test() -> RetFuture<Option<PathBuf>> {
    let panel = Panel::open_panel();
    AsyncDialog::new(panel).into()
}

pub fn callback_test<F: Fn(())>(cb: F) {
    objc::rc::autoreleasepool(|| {
        let panel = Panel::open_panel();

        let app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let was_running: bool = unsafe { msg_send![app, isRunning] };

        let completion = block::ConcreteBlock::new(move |result: i32| {
            if !was_running {
                unsafe {
                    let _: () = msg_send![app, stop: nil];
                }
            }
            cb(());
        });

        unsafe {
            let _: () = msg_send![*panel.panel, beginWithCompletionHandler: &completion];

            if !was_running {
                let _: () = msg_send![app, run];
            }
        }

        std::mem::forget(completion);
    });
}

//
// Internal
//

extern "C" {
    pub fn CGShieldingWindowLevel() -> i32;
}
fn make_nsstring(s: &str) -> id {
    unsafe { NSString::alloc(nil).init_str(s).autorelease() }
}

struct Panel {
    panel: StrongPtr,
    _policy_manager: AppPolicyManager,
    key_window: *mut Object,
}

impl Panel {
    fn new(panel: *mut Object) -> Self {
        let _policy_manager = AppPolicyManager::new();
        let key_window = unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            msg_send![app, keyWindow]
        };

        let _: () = unsafe { msg_send![panel, setLevel: CGShieldingWindowLevel()] };
        Self {
            _policy_manager,
            panel: unsafe { StrongPtr::retain(panel) },
            key_window,
        }
    }

    fn open_panel() -> Self {
        Self::new(unsafe { msg_send![class!(NSOpenPanel), openPanel] })
    }

    fn save_panel() -> Self {
        Self::new(unsafe { msg_send![class!(NSSavePanel), savePanel] })
    }

    fn run_modal(&self) -> i32 {
        unsafe { msg_send![*self.panel, runModal] }
    }

    fn set_can_choose_directories(&self, v: BOOL) {
        let _: () = unsafe { msg_send![*self.panel, setCanChooseDirectories: v] };
    }

    fn set_can_choose_files(&self, v: BOOL) {
        let _: () = unsafe { msg_send![*self.panel, setCanChooseFiles: v] };
    }

    fn set_allows_multiple_selection(&self, v: BOOL) {
        let _: () = unsafe { msg_send![*self.panel, setAllowsMultipleSelection: v] };
    }

    fn add_filters<'a>(&self, params: &FileDialog<'a>) {
        let mut exts: Vec<&str> = Vec::new();

        for filter in params.filters.iter() {
            exts.append(&mut filter.extensions.to_vec());
        }

        unsafe {
            let f_raw: Vec<_> = exts.iter().map(|ext| make_nsstring(ext)).collect();

            let array = NSArray::arrayWithObjects(nil, f_raw.as_slice());
            let _: () = msg_send![*self.panel, setAllowedFileTypes: array];
        }
    }

    fn set_path(&self, path: &Path) {
        if let Some(path) = path.to_str() {
            unsafe {
                let url = NSURL::alloc(nil)
                    .initFileURLWithPath_isDirectory_(make_nsstring(path), YES)
                    .autorelease();
                let () = msg_send![*self.panel, setDirectoryURL: url];
            }
        }
    }

    fn get_result(&self) -> PathBuf {
        unsafe {
            let url: id = msg_send![*self.panel, URL];
            let path: id = msg_send![url, path];
            let utf8: *const i32 = msg_send![path, UTF8String];
            let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

            let slice = std::slice::from_raw_parts(utf8 as *const _, len);
            let result = std::str::from_utf8_unchecked(slice);

            result.into()
        }
    }

    fn get_results(&self) -> Vec<PathBuf> {
        unsafe {
            let urls: id = msg_send![*self.panel, URLs];

            let count = urls.count();

            let mut res = Vec::new();
            for id in 0..count {
                let url = urls.objectAtIndex(id);
                let path: id = msg_send![url, path];
                let utf8: *const i32 = msg_send![path, UTF8String];
                let len: usize = msg_send![path, lengthOfBytesUsingEncoding:4 /*UTF8*/];

                let slice = std::slice::from_raw_parts(utf8 as *const _, len);
                let result = std::str::from_utf8_unchecked(slice);
                res.push(result.into());
            }

            res
        }
    }
}

trait OutputFrom<F> {
    fn from(from: &F, res_id: i32) -> Self;
}

impl OutputFrom<Panel> for Option<PathBuf> {
    fn from(panel: &Panel, res_id: i32) -> Self {
        if res_id == 1 {
            Some(panel.get_result())
        } else {
            None
        }
    }
}

impl OutputFrom<Panel> for Option<Vec<PathBuf>> {
    fn from(panel: &Panel, res_id: i32) -> Self {
        if res_id == 1 {
            Some(panel.get_results())
        } else {
            None
        }
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _: () = unsafe { msg_send![self.key_window, makeKeyAndOrderFront: nil] };

        unsafe {
            let i: i32 = msg_send![*self.panel, retainCount];
            println!("{:?}", std::thread::current().id());
            println!("Will drop, with retain count of: {}", i);
        }
    }
}

pub struct AppPolicyManager {
    initial_policy: i32,
}

impl AppPolicyManager {
    pub fn new() -> Self {
        #[repr(i32)]
        #[derive(Debug, PartialEq)]
        enum ApplicationActivationPolicy {
            //Regular = 0,
            Accessory = 1,
            Prohibited = 2,
            //Error = -1,
        }

        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            let initial_policy: i32 = msg_send![app, activationPolicy];

            if initial_policy == ApplicationActivationPolicy::Prohibited as i32 {
                let new_pol = ApplicationActivationPolicy::Accessory as i32;
                let _: () = msg_send![app, setActivationPolicy: new_pol];
            }

            Self { initial_policy }
        }
    }
}
impl Drop for AppPolicyManager {
    fn drop(&mut self) {
        unsafe {
            let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
            // Restore initial pol
            let _: () = msg_send![app, setActivationPolicy: self.initial_policy];
        }
    }
}
