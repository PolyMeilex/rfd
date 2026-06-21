#[cfg(not(target_family = "wasm"))]
fn main() {
    let path = std::env::current_dir().unwrap();

    let res = rfd::FileDialog::new()
        .add_filter("text", &["txt", "rs"])
        .add_filter("rust", &["rs", "toml"])
        .set_directory(&path)
        .pick_file();

    println!("The user choose: {:#?}", res);
}

#[cfg(target_family = "wasm")]
fn main() {
    // On wasm only async dialogs are possible
}
