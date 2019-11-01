use rocket::response::NamedFile;
use rocket::*;

use crate::Result;

#[get("/")]
pub fn index() -> &'static str {
    // TODO: Expose static files for a download webpage or such.
    "Welcome to the index! You probably want to visit /windows or /linux"
}

#[get("/robots.txt")]
pub fn robots() -> &'static str {
    "User-agent: *
     Disallow: /"
}

#[get("/favicon.ico")]
pub fn favicon() -> Result<NamedFile> {
    Ok(NamedFile::open(format!(
        "{}/favicon.ico",
        crate::CONFIG.static_files
    ))?)
}
