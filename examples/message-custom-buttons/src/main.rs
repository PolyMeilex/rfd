fn main() {
    #[cfg(not(feature = "gtk3"))]
    let res = "";

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        all(
            any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
                target_os = "openbsd"
            ),
            feature = "gtk3"
        )
    ))]
    let res = rfd::MessageDialog::new()
        .set_title("Msg!")
        .set_description("Description!")
        .set_level(rfd::MessageLevel::Warning)
        .set_buttons(rfd::MessageButtons::OkCancelCustom("Got it!".to_string(), "No!".to_string()))
        .show();

    println!("{}", res);
}
