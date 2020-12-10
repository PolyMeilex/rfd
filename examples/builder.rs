use rfd::{Dialog, Response};

fn main() {
    let res = Dialog::pick_file()
        .filter("text", &["txt", "rs"])
        .filter("rust", &["rs", "toml"])
        .starting_directory(&"/")
        .open();

    match res {
        Response::Single(path) => {
            println!("The user choose this file: {}", path.to_string_lossy());
        }
        _ => {
            println!("The user did not choose any file, or an error occured!");
        }
    }

    let res = Dialog::pick_files()
        .filter("text", &["txt", "rs"])
        .filter("rust", &["rs", "toml"])
        .starting_directory(&"/")
        .open();

    match res {
        Response::Multiple(path) => {
            println!("The user choose those files: {:#?}", path);
        }
        _ => {
            println!("The user did not choose any file, or an error occured!");
        }
    }
}
