use rfd::Dialog;

fn main() {
    let res = Dialog::pick_files()
        .filter("text", &["txt", "rs"])
        .filter("rust", &["rs", "toml"])
        .starting_directory(&"/")
        .open();

    println!("The user choose: {:#?}", res);
}
