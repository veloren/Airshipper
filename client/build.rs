#[cfg(windows)]
fn main() {
    //Set executable logo with winres here:
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icons/logo.ico");
    res.compile().expect("failed to build executable logo.");
}

#[cfg(unix)]
fn main() {}
