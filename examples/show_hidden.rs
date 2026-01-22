//! Example demonstrating the show_hidden_files option.
//!
//! Run with: cargo run --example show_hidden

use rfd::FileDialog;

fn main() {
    // Open a file dialog with hidden files shown
    let file = FileDialog::new()
        .set_title("Select a file (hidden files visible)")
        .set_show_hidden_files(true)
        .pick_file();

    match file {
        Some(path) => println!("Selected: {}", path.display()),
        None => println!("No file selected"),
    }
}
