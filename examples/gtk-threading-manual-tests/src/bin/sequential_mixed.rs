//! Mix of sync and async dialogs from different threads, one after another
use threading_tests::*;

fn main() {
    init();
    message("main 1");
    futures::executor::block_on(message_async("main 2"));
    spawn("worker-a", || message("worker a")).join().unwrap();
    spawn("worker-b", || futures::executor::block_on(message_async("worker b")))
        .join()
        .unwrap();
    message("main 3");
}
