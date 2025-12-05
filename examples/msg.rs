fn main() {
    let res = rfd::MessageDialog::new()
        .set_title("Msg!")
        .set_description("Description!")
        .set_buttons(rfd::MessageButtons::OkCancel)
        .set_level(rfd::MessageLevel::Error)
        .show();
    println!("res: {res}");

    futures::executor::block_on(async move {
        let res = rfd::AsyncMessageDialog::new()
            .set_title("Msg!")
            .set_description("Description!")
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show()
            .await;
        println!("res: {res}");
    });
}
