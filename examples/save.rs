#[cfg(not(target_family = "wasm"))]
fn main() {
    let path = std::env::current_dir().unwrap();

    let res = rfd::FileDialog::new()
        .set_file_name("foo.txt")
        .set_directory(&path)
        .save_file();

    println!("The user choose: {:#?}", res);
}

#[cfg(target_family = "wasm")]
fn main() {
    // On wasm only async dialogs are possible
}
