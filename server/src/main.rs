#![allow(clippy::unit_arg)]

// How to send manual webhooks:
// curl --header "Content-Type: application/json" --request POST --data "@<FILE_WITH_WEBHOOK_DATA>" --header "X-Gitlab-Event: Pipeline Hook" --header "X-Gitlab-Token: <TOKEN>" http://<ADDRESS>
mod logger;

#[rocket::launch]
async fn rocket() -> rocket::Rocket {
    dotenv::from_path("server/.env").ok();
    logger::init();
    airshipper_server::rocket().await.unwrap()
}
