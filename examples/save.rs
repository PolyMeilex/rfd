use rfd::DialogOptions;

const FILTERS: &[(&str, &str)] = &[(".txt", "*.txt"), (".rs", "*.rs")];

fn main() {
    let params = DialogOptions::new().set_filters(FILTERS);
    let path = rfd::save_file(params);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any path, or an error occured!".to_owned(),
            |path| format!("The user choose this path: {}", path.to_string_lossy())
        )
    );
}
