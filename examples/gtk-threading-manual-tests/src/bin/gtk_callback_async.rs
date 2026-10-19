//! App runs its own GTK main loop and awaits async dialogs on the GTK thread itself
use threading_tests::*;

fn main() {
    init();
    gtk::init();
    gtk::run_loop(|main_loop| {
        glib::MainContext::default().spawn_local(async move {
            pick_file_async("gtk spawn_local").await;
            message_async("gtk spawn_local").await;
            main_loop.quit();
        });
    });
}
