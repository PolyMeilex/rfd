//! App runs its own GTK main loop on main thread, a worker thread opens sync dialogs.
//! They should be shown by the app's main loop.
use threading_tests::*;

fn main() {
    init();
    gtk::init();
    gtk::run_loop(|main_loop| {
        spawn("worker", move || {
            pick_file("worker");
            message("worker");
            main_loop.quit();
        });
    });
}
