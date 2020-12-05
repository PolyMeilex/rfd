use rfd::DialogParams;

const FILTERS: &[(&str, &str)] = &[(".txt", "*.txt"), (".rs", "*.rs")];

fn main() {
    let params = DialogParams::new().set_filters(FILTERS);
    let path = rfd::save_file_with_params(params);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any path, or an error occured!".to_owned(),
            |path| format!("The user choose this path: {}", path.to_string_lossy())
        )
    );
}
