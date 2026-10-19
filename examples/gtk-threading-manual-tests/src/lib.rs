use std::io::Write;
use std::time::Instant;

use log::info;

pub use glib;

pub fn init() {
    let start = Instant::now();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(move |buf, record| {
            let thread = std::thread::current();
            writeln!(
                buf,
                "[{:>8.3}s] [{} {:?}] {} {}: {}",
                start.elapsed().as_secs_f64(),
                thread.name().unwrap_or("unnamed"),
                thread.id(),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}

pub fn spawn<T: Send + 'static>(
    name: &str,
    f: impl FnOnce() -> T + Send + 'static,
) -> std::thread::JoinHandle<T> {
    std::thread::Builder::new()
        .name(name.into())
        .spawn(f)
        .unwrap()
}

pub fn pick_file(label: &str) -> Option<std::path::PathBuf> {
    info!("{label}: opening sync file dialog");
    let res = rfd::FileDialog::new()
        .set_title(format!("[sync] {label}"))
        .pick_file();
    info!("{label}: sync file dialog returned {res:?}");
    res
}

pub fn message(label: &str) -> rfd::MessageDialogResult {
    info!("{label}: opening sync message dialog");
    let res = rfd::MessageDialog::new()
        .set_title(format!("[sync] {label}"))
        .set_description("Press any button")
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show();
    info!("{label}: sync message dialog returned {res:?}");
    res
}

pub async fn pick_file_async(label: &str) -> Option<rfd::FileHandle> {
    info!("{label}: opening async file dialog");
    let res = rfd::AsyncFileDialog::new()
        .set_title(format!("[async] {label}"))
        .pick_file()
        .await;
    info!(
        "{label}: async file dialog returned {:?}",
        res.as_ref().map(|f| f.path().to_owned())
    );
    res
}

pub async fn message_async(label: &str) -> rfd::MessageDialogResult {
    info!("{label}: opening async message dialog");
    let res = rfd::AsyncMessageDialog::new()
        .set_title(format!("[async] {label}"))
        .set_description("Press any button")
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show()
        .await;
    info!("{label}: async message dialog returned {res:?}");
    res
}

/// Helpers for simulating an application that uses GTK itself
pub mod gtk {
    use log::info;

    pub fn init() {
        info!("app: initializing GTK");
        #[cfg(feature = "gtk4")]
        gtk4::init().unwrap();
        #[cfg(feature = "gtk3")]
        unsafe {
            gtk_sys::gtk_init(std::ptr::null_mut(), std::ptr::null_mut())
        };
        #[cfg(not(any(feature = "gtk3", feature = "gtk4")))]
        panic!("this test requires `gtk3` or `gtk4` feature");
    }

    /// Initializes GTK through C API
    pub fn init_c() {
        info!("app: initializing GTK through C API");
        #[cfg(feature = "gtk4")]
        unsafe {
            gtk4::ffi::gtk_init()
        };
        #[cfg(feature = "gtk3")]
        unsafe {
            gtk_sys::gtk_init(std::ptr::null_mut(), std::ptr::null_mut())
        };
        #[cfg(not(any(feature = "gtk3", feature = "gtk4")))]
        panic!("this test requires `gtk3` or `gtk4` feature");
    }

    pub fn run_loop(f: impl FnOnce(glib::MainLoop) + 'static) {
        let main_loop = glib::MainLoop::new(None, false);
        let l = main_loop.clone();
        glib::idle_add_local_once(move || f(l));
        info!("app: running main loop");
        main_loop.run();
        info!("app: main loop exited");
    }
}
