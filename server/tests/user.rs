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
        Some("Welcome to the index! You probably want to visit /windows or /linux".to_string())
    );
}
