use rfd::Dialog;

fn main() {
    let res = Dialog::pick_files()
        .filter("text", &["txt", "rs"])
        .filter("rust", &["rs", "toml"])
        .starting_directory(&"/")
        .open();

    let _file = res.first();
    println!("The user choose: {:#?}", res);
}
