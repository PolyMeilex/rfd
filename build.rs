fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("target OS not detected");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("target ARCH not detected");

    match (target_arch.as_str(), target_os.as_str()) {
        (_, "macos") => println!("cargo:rustc-link-lib=framework=AppKit"),
        (_, "windows") => {}
        ("wasm32", _) => {}
        _ => {
            let gtk = std::env::var_os("CARGO_FEATURE_GTK3").is_some();
            let xdg = std::env::var_os("CARGO_FEATURE_XDG_PORTAL").is_some();

            if gtk && xdg {
                panic!("You can't enable both `gtk3` and `xdg-portal` features at once");
            } else if !gtk && !xdg {
                panic!("You need to choose at least one backend: `gtk3` or `xdg-portal` features for {target_arch}-{target_os}");
            }
        }
    }
}
