fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("target OS not detected");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("target ARCH not detected");
    let target_family =
        std::env::var("CARGO_CFG_TARGET_FAMILY").expect("target FAMILY not detected");
    let is_wasm = target_family.split(',').any(|f| f == "wasm");

    println!("cargo:rustc-check-cfg=cfg(rfd_unixlike)");
    // Keep in sync with Cargo.toml
    if matches!(
        target_os.as_str(),
        "linux" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" | "solaris" | "illumos"
    ) {
        println!("cargo:rustc-cfg=rfd_unixlike");
    }

    match target_os.as_str() {
        "macos" => println!("cargo:rustc-link-lib=framework=AppKit"),
        "windows" => {}
        _ if is_wasm => {}
        _ => {
            let backends = ["GTK3", "GTK4", "XDG_PORTAL"]
                .iter()
                .filter(|f| std::env::var_os(format!("CARGO_FEATURE_{f}")).is_some())
                .count();

            if backends != 1 {
                panic!("You need to enable exactly one backend feature: `gtk3`, `gtk4` or `xdg-portal` for {target_arch}-{target_os}");
            }
        }
    }
}
