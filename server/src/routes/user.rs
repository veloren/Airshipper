use rocket::*;

#[get("/")]
pub fn index() -> &'static str {
    "You probably want to visit /latest/windows or /latest/linux"
}

#[get("/ping")]
pub fn ping() -> &'static str {
    ""
}

#[get("/robots.txt")]
pub fn robots() -> &'static str {
    "User-agent: *
     Disallow: /"
}

#[get("/favicon.ico")]
pub fn favicon() {}
