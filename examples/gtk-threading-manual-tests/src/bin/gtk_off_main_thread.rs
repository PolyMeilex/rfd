//! App initializes GTK and runs its main loop on a non-main thread, while main thread opens
//! sync and async dialogs
use std::sync::mpsc;
use threading_tests::*;

fn main() {
    init();
    let (tx, rx) = mpsc::channel();
    let (quit_tx, quit_rx) = mpsc::channel::<glib::MainLoop>();
    let gtk = spawn("app-gtk", move || {
        gtk::init();
        gtk::run_loop(move |main_loop| {
            quit_tx.send(main_loop).unwrap();
            tx.send(()).unwrap();
        });
    });
    rx.recv().unwrap();

    message("main, GTK on other thread");
    futures::executor::block_on(message_async("main, GTK on other thread"));

    quit_rx.recv().unwrap().quit();
    gtk.join().unwrap();
}
