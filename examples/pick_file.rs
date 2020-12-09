fn main() {
    let path = rfd::pick_file(None);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any file, or an error occured!".to_owned(),
            |path| format!("The user choose this file: {}", path.to_string_lossy())
        )
    );
}
