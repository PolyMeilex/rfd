use ::std::io::{ Read, self };
use ::futures::executor::block_on;
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
        .set_buttons(rfd::MessageButtons::OkCancel)
        .set_level(rfd::MessageLevel::Error)
        .show();
    println!("被点击按钮是 {}。敲击 Ctrl+D 继续。", res);
    let mut stdin = io::stdin();
    let mut buffer: Vec<u8> = vec![];
    stdin.read_to_end(&mut buffer).unwrap();
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
    block_on(async move {
        let res = rfd::AsyncMessageDialog::new()
            .set_title("Msg!")
            .set_description("Description!")
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show().await;
        println!("被点击按钮是 {}", res);
    });
    println!("敲击 Ctrl+D 继续。");
    let mut stdin = io::stdin();
    let mut buffer: Vec<u8> = vec![];
    stdin.read_to_end(&mut buffer).unwrap();
    println!("结束");
}