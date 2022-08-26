#![allow(clippy::unit_arg)]

// How to send manual webhooks:
// curl --header "Content-Type: application/json" --request POST --data "@<FILE_WITH_WEBHOOK_DATA>" --header "X-Gitlab-Event: Pipeline Hook" --header "X-Gitlab-Token: <TOKEN>" http://<ADDRESS>

use rocket::{fs::FileServer, *};
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

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
use config::{loading, Config, CONFIG_PATH, LOCAL_STORAGE_PATH};
use metrics::Metrics;
use std::{path::Path, sync::Arc};

pub type Result<T> = std::result::Result<T, ServerError>;
pub use db::{DbConnection, FsStorage};

lazy_static::lazy_static! {
    /// Contains all configuration needed.
    pub static ref CONFIG: Config = Config::compile(loading::Config::load(Path::new(CONFIG_PATH)).unwrap_or_else(|_| panic!("Couldn't open config file {}", CONFIG_PATH))).unwrap();
}

#[rocket::launch]
async fn rocket() -> _ {
    dotenv::from_path("server/.env").ok();
    build().await.unwrap()
}

#[allow(clippy::nonstandard_macro_braces)]
async fn build() -> Result<rocket::Rocket<rocket::Build>> {
    let guard = Arc::new(logger::init(tracing::metadata::LevelFilter::INFO));

    let local_storage_folder = CONFIG.get_local_storage_path();
    if !local_storage_folder.exists() {
        tokio::fs::create_dir_all(local_storage_folder.clone())
            .await
            .unwrap();
    }

    let metrics = Metrics::new().expect("Failed to create prometheus statistics!");
    let metrics = Arc::new(metrics);

    // Base of the config and attach everything else
    Ok(CONFIG
        .rocket()
        .attach(DbConnection::fairing())
        .attach(fairings::db::DbInit)
        .attach(metrics.clone())
        .manage(guard)
        .manage(metrics)
        .mount("/", routes![
            routes::gitlab::post_pipeline_update,
            routes::user::index,
            routes::user::ping,
            routes::user::robots,
            routes::user::favicon,
            routes::api::announcement,
            routes::api::api_version,
            routes::api::channels,
            routes::api::version,
            routes::api::download,
            routes::metrics::metrics,
        ])
        .mount(
            &format!("/{}", LOCAL_STORAGE_PATH),
            FileServer::from(local_storage_folder),
        )
        .register("/", catchers![routes::catchers::not_found]))
}
