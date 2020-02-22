use rocket::response::NamedFile;
use rocket::*;

use crate::Result;

#[get("/")]
pub fn index() -> &'static str {
    "You probably want to visit /latest/windows or /latest/linux"
}

#[get("/robots.txt")]
pub fn robots() -> &'static str {
    "User-agent: *
     Disallow: /"
}

#[get("/favicon.ico")]
pub fn favicon() -> Result<NamedFile> {
    Ok(NamedFile::open("favicon.ico")?)
}
