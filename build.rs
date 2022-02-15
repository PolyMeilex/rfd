fn main() {
    let target = std::env::var("TARGET").unwrap();

    if target.contains("darwin") {
        println!("cargo:rustc-link-lib=framework=AppKit");
    }

    #[cfg(all(feature = "gtk3", feature = "xdg-portal"))]
    compile_error!("You can't enable both GTK3 & XDG Portal features at once");

    #[cfg(not(any(feature = "gtk3", feature = "xdg-portal")))]
    compile_error!("You need to choose at least one backend: `gtk3` or `xdg-portal` features");
}
