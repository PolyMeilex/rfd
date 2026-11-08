//! Run with `cargo run --example macos_native_events` from a terminal on macOS.

#[cfg(target_os = "macos")]
use std::{
    error::Error,
    io::{self, Write},
};

#[cfg(target_os = "macos")]
use rfd::{macos::run_with_native_events, FileDialog};

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn Error>> {
    let work_result = run_with_native_events(|| -> io::Result<()> {
        let mut command = String::new();
        loop {
            // Console input waits on this worker. Main keeps processing native
            // events after each dialog returns, including dismissal animation.
            print!("Choose file, folder, or quit: ");
            io::stdout().flush()?;
            command.clear();
            if io::stdin().read_line(&mut command)? == 0 {
                return Ok(());
            }

            let selection = match command.trim() {
                "file" => FileDialog::new().pick_file(),
                "folder" => FileDialog::new().pick_folder(),
                "quit" => return Ok(()),
                _ => {
                    println!("Enter file, folder, or quit.");
                    continue;
                }
            };
            println!("Selection: {selection:?}");
        }
    })?;
    work_result?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("This example is supported only on macOS.");
}
