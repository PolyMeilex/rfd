//! Async dialogs created and awaited on the main thread with a plain executor
use threading_tests::*;

fn main() {
    init();
    futures::executor::block_on(async {
        pick_file_async("block_on").await;
        message_async("block_on").await;
    });
}
