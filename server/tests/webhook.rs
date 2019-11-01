use rocket::http::{ContentType, Header, Status};
use rocket::local::Client;

use server::rocket;

// TODO: Finish

#[test]
fn recv_pipeline_hook() {
    let client = Client::new(rocket()).expect("invalid rocket instance");

    // Valid post
    let mut request = client.post("/");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new("X-Gitlab-Event", server::config::HOOK_TYPE));
    request.add_header(Header::new("X-Gitlab-Token", "gitlab_secret_test"));
    request.set_body(include_str!("webhook/artifacts.json"));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Invalid secret
    let mut request = client.post("/");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new("X-Gitlab-Event", server::config::HOOK_TYPE));
    request.add_header(Header::new("X-Gitlab-Token", "invalid_secret"));
    request.set_body(include_str!("webhook/artifacts.json"));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Unauthorized);

    // Invalid Event Type
    let mut request = client.post("/");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new("X-Gitlab-Event", "Job Hook"));
    request.add_header(Header::new("X-Gitlab-Token", "gitlab_secret_test"));
    request.set_body(include_str!("webhook/artifacts.json"));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);

    // valid but empty post
    let mut request = client.post("/");
    request.add_header(ContentType::JSON);
    request.add_header(Header::new("X-Gitlab-Event", server::config::HOOK_TYPE));
    request.add_header(Header::new("X-Gitlab-Token", "gitlab_secret_test"));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}
