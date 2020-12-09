fn main() {
    let path = rfd::pick_files(None);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any file, or an error occured!".to_owned(),
            |path| format!("The user choose this files: {:#?}", path)
        )
    );
}
