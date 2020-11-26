use rfd::file_dialog::{self, DialogParams};

fn main() {
    let params = DialogParams {
        filters: &[(".mid", "*.mid"), (".midi", "*.midi")],
    };

    let path = file_dialog::open(params);

    println!("{:?}", path);
}
