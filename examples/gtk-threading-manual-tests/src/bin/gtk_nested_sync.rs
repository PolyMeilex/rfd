//! Sync dialog from a GTK callback, and while it is open (nested main loop) a timer opens
//! another sync dialog, nesting a main loop inside of a nested main loop
use std::time::Duration;
use threading_tests::*;

fn main() {
    init();
    gtk::init();
    gtk::run_loop(|main_loop| {
        glib::timeout_add_local_once(Duration::from_secs(2), || {
            message("timer, while file dialog is open");
        });
        pick_file("outer");
        main_loop.quit();
    });
}
