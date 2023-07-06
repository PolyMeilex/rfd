fn main() {
    let res = rfd::MessageDialog::new()
        .set_title("Msg!")
        .set_description("Description!")
        .set_level(rfd::MessageLevel::Warning)
        .set_buttons(rfd::MessageButtons::OkCancelCustom(
            "Got it!".to_string(),
            "No!".to_string(),
        ))
        .show();
    println!("{res}");

    let res = rfd::MessageDialog::new()
        .set_title("Do you want to save the changes you made?")
        .set_description("Your changes will be lost if you don't save them.")
        .set_level(rfd::MessageLevel::Warning)
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
            "Save".to_string(),
            "Don't Save".to_string(),
            "Cancel".to_string(),
        ))
        .show();
    println!("{res}");
}
