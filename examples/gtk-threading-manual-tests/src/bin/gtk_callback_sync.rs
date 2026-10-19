//! App runs its own GTK main loop and opens sync dialogs from a GTK callback, on the GTK thread
use threading_tests::*;

fn main() {
    init();
    gtk::init();
    gtk::run_loop(|main_loop| {
        pick_file("gtk callback");
        message("gtk callback");
        main_loop.quit();
    });
}
