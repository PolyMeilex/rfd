//! Sync dialogs from the main thread of an app that knows nothing about GTK
use threading_tests::*;

fn main() {
    init();
    pick_file("main thread");
    message("main thread");
}
