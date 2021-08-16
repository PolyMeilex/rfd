fn main() {
    let target = std::env::var("TARGET").unwrap();

    if target.contains("darwin") {
        println!("cargo:rustc-link-lib=framework=AppKit");
    }

    // if std::env::var("TARGET").unwrap().contains("-ios") {
    //     println!("cargo:rustc-link-lib=framework=UIKit");
    // } else {
    //     println!("cargo:rustc-link-lib=framework=AppKit");
    // }
}
