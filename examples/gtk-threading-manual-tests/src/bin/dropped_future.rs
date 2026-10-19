//! Async dialog future dropped without being awaited, following dialogs should still work
use threading_tests::*;

fn main() {
    init();
    let task = rfd::AsyncMessageDialog::new()
        .set_title("[async] dropped future")
        .set_description("The future of this dialog was dropped (it may or may not be shown)")
        .show();
    drop(task);
    log::info!("main: dropped the future");

    message("after drop");
    futures::executor::block_on(message_async("after drop"));
}
