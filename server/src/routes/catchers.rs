use rocket::*;

#[catch(404)]
pub fn not_found(req: &Request) -> String {
    format!("Couldn't find '{}'", req.uri())
}
