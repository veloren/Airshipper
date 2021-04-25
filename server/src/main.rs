#![allow(clippy::unit_arg)]

mod config;
mod db;
mod error;
mod fairings;
mod guards;
mod logger;
mod metrics;
mod models;
mod prune;
mod routes;
mod webhook;

use crate::error::ServerError;
use config::ServerConfig;
use db::ROOT_FOLDER;
use metrics::Metrics;
use rocket::*;
use rocket_contrib::serve::StaticFiles;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, ServerError>;
pub use db::{DbConnection, FsStorage};

lazy_static::lazy_static! {
    /// Contains all configuration needed.
    pub static ref CONFIG: ServerConfig = ServerConfig::load();
}

// How to send manual webhooks:
// LINUX: curl --header "Content-Type: application/json" --request POST --data "@<FILE_WITH_WEBHOOK_DATA>" --header "X-Gitlab-Event: Pipeline Hook" --header "X-Gitlab-Token: <TOKEN>" http://<ADDRESS>
// POWERSHELL: curl.exe --header "Content-Type: application/json" --request POST --data "@<FILE_WITH_WEBHOOK_DATA>" --header "X-Gitlab-Event: Pipeline Hook" --header "X-Gitlab-Token: <TOKEN>" http://<ADDRESS>

#[rocket::launch]
async fn rocket() -> _ {
    let root_folder = PathBuf::from(ROOT_FOLDER);
    if !root_folder.exists() {
        tokio::fs::create_dir_all(root_folder).await.unwrap();
    }

    dotenv::from_path("server/.env").ok();
    dotenv::from_path(".env").ok();

    // Access the global config for lazy_static to lazy load the config.
    let _ = CONFIG.db_path;

    logger::init();
    rocket::build()
        .attach(fairings::DbInit)
        .manage(Metrics::new().expect("Failed to create prometheus statistics!"))
        .mount("/", routes![
            routes::gitlab::post_pipeline_update,
            routes::user::index,
            routes::user::robots,
            routes::user::favicon,
            routes::api::version,
            routes::api::channel_version,
            routes::api::download,
            routes::api::channel_download,
            routes::metrics::metrics,
        ])
        .mount("/nightly", StaticFiles::from(db::fs::ROOT_FOLDER))
        .register("/", catchers![routes::catchers::not_found])
}
