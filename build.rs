fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("target OS not detected");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("target ARCH not detected");
    let target_family =
        std::env::var("CARGO_CFG_TARGET_FAMILY").expect("target FAMILY not detected");
    let is_wasm = target_family.split(',').any(|f| f == "wasm");

    match target_os.as_str() {
        "macos" => println!("cargo:rustc-link-lib=framework=AppKit"),
        "windows" => {}
        _ if is_wasm => {}
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
