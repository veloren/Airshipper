use winresource::WindowsResource;

fn main() {
    // #[cfg(target_os = "windows")] does not work in build.rs for cross-compilation
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "windows" {
        return;
    }

    //Set executable logo with winres here:
    let mut res = WindowsResource::new();
    let icon_path = "assets/icons/logo.ico";

    res.set_icon(icon_path);
    res.compile().expect("failed to build executable logo.");
}
