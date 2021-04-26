#![allow(clippy::unit_arg)]

use std::path::PathBuf;

use db::ROOT_FOLDER;
use rocket::*;
use rocket_contrib::serve::StaticFiles;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod config;
mod db;
mod error;
mod fairings;
mod guards;
mod metrics;
mod models;
mod prune;
mod routes;
mod webhook;

use crate::error::ServerError;
use config::ServerConfig;
use metrics::Metrics;

pub type Result<T> = std::result::Result<T, ServerError>;
pub use db::{DbConnection, FsStorage};

lazy_static::lazy_static! {
    /// Contains all configuration needed.
    pub static ref CONFIG: ServerConfig = ServerConfig::load();
}

pub async fn rocket() -> Result<rocket::Rocket> {
    let root_folder = PathBuf::from(ROOT_FOLDER);
    if !root_folder.exists() {
        tokio::fs::create_dir_all(root_folder).await.unwrap();
    }

    // Base of the config and attach everything else
    Ok(CONFIG
        .rocket()
        .attach(DbConnection::fairing())
        .attach(fairings::db::DbInit)
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
        .register(catchers![routes::catchers::not_found]))
}
