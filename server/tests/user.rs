use rocket::http::Status;
use rocket::local::Client;

use server::rocket;

// TODO: Finish

#[test]
fn index() {
    let client = Client::new(rocket()).expect("invalid rocket instance");

    let mut response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("You probably want to visit /latest/windows or /latest/linux".to_string())
    );
}
