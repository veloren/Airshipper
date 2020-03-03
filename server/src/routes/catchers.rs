use rocket::*;

#[catch(404)]
pub fn not_found(req: &Request) -> String {
    format!("Couldn't find '{}'", req.uri())
}

#[catch(500)]
pub fn internal_error(req: &Request) -> String {
    format!(
        "We hit a serious error with your request to '{}'. Please report this to @Songtronix#4790 on Discord!",
        req.uri()
    )
}
