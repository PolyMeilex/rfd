fn main() {
    let target = std::env::var("TARGET").unwrap();

    if target.contains("darwin") {
        println!("cargo:rustc-link-lib=framework=AppKit");
    }

    // println!("cargo:rustc-link-lib=framework=Foundation");
    // if std::env::var("TARGET").unwrap().contains("-ios") {
    //     println!("cargo:rustc-link-lib=framework=UIKit");
    // } else {
    //     println!("cargo:rustc-link-lib=framework=AppKit");
    // }

    // println!("cargo:rustc-link-lib=framework=CoreGraphics");
    // println!("cargo:rustc-link-lib=framework=QuartzCore");

    // println!("cargo:rustc-link-lib=framework=Security");

    // #[cfg(feature = "webview")]
    // println!("cargo:rustc-link-lib=framework=WebKit");
    // #[cfg(feature = "cloudkit")]
    // println!("cargo:rustc-link-lib=framework=CloudKit");

    // #[cfg(feature = "user-notifications")]
    // println!("cargo:rustc-link-lib=framework=UserNotifications");
}
