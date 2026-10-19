//! Several async dialogs open at the same time, awaited together with `join`
use threading_tests::*;

fn main() {
    init();
    futures::executor::block_on(async {
        futures::join!(
            message_async("first"),
            pick_file_async("second"),
            message_async("third"),
        );
    });
}
