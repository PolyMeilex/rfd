//! Sync dialogs from a worker thread, main thread just waits
use threading_tests::*;

fn main() {
    init();
    spawn("worker", || {
        pick_file("worker");
        message("worker");
    })
    .join()
    .unwrap();
}
