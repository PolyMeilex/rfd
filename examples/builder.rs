use rfd::Dialog;

fn main() {
    let res = Dialog::pick_files()
        .add_filter("text", &["txt", "rs"])
        .add_filter("rust", &["rs", "toml"])
        .starting_directory(&"/")
        .open();

    let _file = res.first();
    println!("The user choose: {:#?}", res);
}
