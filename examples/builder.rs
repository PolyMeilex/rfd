use rfd::FileDialog;

fn main() {
    let res = FileDialog::new()
        .add_filter("text", &["txt", "rs"])
        .add_filter("rust", &["rs", "toml"])
        .set_directory(&"/")
        .pick_files();

    println!("The user choose: {:#?}", res);
}
