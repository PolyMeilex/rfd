use rfd::DialogParams;

const FILTERS: &[(&str, &str)] = &[(".txt", "*.txt"), (".rs", "*.rs")];

fn main() {
    let params = DialogParams::new()
        .set_filters(FILTERS)
        .set_starting_directory("/");

    let path = rfd::open_file_with_params(params);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any file, or an error occured!".to_owned(),
            |path| format!("The user choose this file: {}", path.to_string_lossy())
        )
    );
}
