use rfd::DialogParams;

const FILTERS: &[(&str, &str)] = &[(".txt", "*.txt")];

fn main() {
    let params = DialogParams::new().set_filters(FILTERS);
    let path = rfd::open_with_params(params);

    println!(
        "{}",
        path.map_or_else(
            || "The user did not choose any file, or an error occured!".to_owned(),
            |path| format!("The user chose this file: {}", path.to_string_lossy())
        )
    );
}
