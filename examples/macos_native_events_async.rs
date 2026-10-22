//! Run with `cargo run --example macos_native_events_async` from a terminal on macOS.
//!
//! The helper processes native events; the executor drives the async API.
//! Without a parent window, constructing the dialog future can block until dismissal.

#[cfg(target_os = "macos")]
use std::{
    error::Error,
    io::{self, Write},
};

#[cfg(target_os = "macos")]
use futures::executor::block_on;
#[cfg(target_os = "macos")]
use rfd::{macos::run_with_native_events, AsyncFileDialog};

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn Error>> {
    let work_result = run_with_native_events(|| -> io::Result<()> {
        let mut command = String::new();
        loop {
            // Console input waits on this worker while main processes native
            // events between dialogs, including deferred dismissal.
            print!("Press Enter to select a file, or enter quit: ");
            io::stdout().flush()?;
            command.clear();
            if io::stdin().read_line(&mut command)? == 0 || command.trim() == "quit" {
                return Ok(());
            }

            // Finish the dialog future before accepting another command or exiting.
            block_on(async {
                match AsyncFileDialog::new().pick_file().await {
                    Some(file) => println!("Selection: {:?}", file.path()),
                    None => println!("No file selected."),
                }
            });
        }
    })?;
    work_result?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("This example is supported only on macOS.");
}
