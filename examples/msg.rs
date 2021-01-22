fn main() {
    let res = rfd::MessageDialog::new()
        .set_text("Msg!")
        .set_buttons(rfd::MessageButtons::OkCancle)
        .show();
}
