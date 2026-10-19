//! App initializes GTK, but opens a sync dialog before it starts its main loop.
//! Pass `--c` to initialize through the C API instead.
//! KNOWN: may hang, as nothing iterates the main context at that point (unless GTK init left
//! the context acquired by this thread, like gtk4-rs `gtk4::init` does)
use threading_tests::*;

fn main() {
    init();
    if std::env::args().any(|a| a == "--c") {
        gtk::init_c();
    } else {
        gtk::init();
    }
    message("before main loop");
    gtk::run_loop(|main_loop| {
        message("inside main loop");
        main_loop.quit();
    });
}
