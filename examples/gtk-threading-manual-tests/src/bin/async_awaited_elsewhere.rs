//! Async dialog future created on main thread, but awaited on another thread
use threading_tests::*;

fn main() {
    init();
    let task = rfd::AsyncFileDialog::new()
        .set_title("[async] created on main, awaited on worker")
        .pick_file();
    log::info!("main: future created, sending it to worker");

    spawn("worker", move || {
        let res = futures::executor::block_on(task);
        log::info!("worker: got {:?}", res.map(|f| f.path().to_owned()));
    })
    .join()
    .unwrap();
}
