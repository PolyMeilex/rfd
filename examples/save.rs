fn main() {
    let path = rfd::save_file(None);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any path, or an error occured!".to_owned(),
            |path| format!("The user choose this path: {}", path.to_string_lossy())
        )
    );
}
