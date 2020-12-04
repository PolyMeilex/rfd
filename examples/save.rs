use rfd::DialogParams;

fn main() {
    let path = rfd::save_file_with_params(DialogParams::new());

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any path, or an error occured!".to_owned(),
            |path| format!("The user chose this path: {}", path.to_string_lossy())
        )
    );
}
