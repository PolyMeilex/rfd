//! App initializes GTK through the C API (bypassing gtk-rs), then uses dialogs both from GTK
//! callback and from a worker thread
use threading_tests::*;

fn main() {
    init();
    gtk::init_c();
    gtk::run_loop(|main_loop| {
        message("gtk callback");
        spawn("worker", move || {
            message("worker");
            main_loop.quit();
        });
    });
}
