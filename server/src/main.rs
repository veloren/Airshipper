#![allow(clippy::unit_arg)]

// How to send manual webhooks:
// curl --header "Content-Type: application/json" --request POST --data "@<FILE_WITH_WEBHOOK_DATA>" --header "X-Gitlab-Event: Pipeline Hook" --header "X-Gitlab-Token: <TOKEN>" http://<ADDRESS>
mod logger;

fn main() {
    dotenv::dotenv().ok();
    logger::init();
    server::rocket().launch().expect("Server failed!");
}
