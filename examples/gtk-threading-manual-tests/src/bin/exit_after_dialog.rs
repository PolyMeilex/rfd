//! Process exits right after a dialog returns, while the GTK thread may still be cleaning up.
//! Racy, run it in a loop: `for i in $(seq 10); do cargo run ... ; echo $?; done`
use threading_tests::*;

fn main() {
    init();
    message("exit right after");
    futures::executor::block_on(message_async("exit right after"));
    log::info!("main: returning from main");
}
