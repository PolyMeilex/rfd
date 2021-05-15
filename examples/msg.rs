fn main() {
    let res = rfd::MessageDialog::new()
        .set_title("Msg!")
        .set_description("Description!")
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show();

    println!("{}", res);
    // println!("{}", futures::executor::block_on(res));
}
