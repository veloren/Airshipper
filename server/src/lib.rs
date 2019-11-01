#![feature(proc_macro_hygiene, decl_macro)]
use rocket::*;
use rocket_contrib::serve::StaticFiles;

pub mod config;
mod db;
mod error;
mod fairings;
mod guards;
mod models;
mod params;
mod routes;
mod webhook;

use crate::error::ServerError;
use config::ServerConfig;

pub type Result<T> = std::result::Result<T, ServerError>;

lazy_static::lazy_static! {
    /// Contains all configuration needed.
    pub static ref CONFIG: ServerConfig = ServerConfig::load();
}

// TODO: * Return status code and error description in json
// TODO: * Avoid duplicated entries in database
// TODO: * Serve better index (download page)
// TODO: * dedicated statistics route with showcasing some db data (how many version it has etc.)

pub fn rocket() -> rocket::Rocket {
    // Base of the config and attach everything else
    CONFIG
        .rocket()
        .attach(fairings::DbInit::default())
        .attach(db::DbConnection::fairing())
        //.attach(fairings::Statistics::default())
        .mount(
            &format!("/{}", CONFIG.static_files),
            StaticFiles::from(&CONFIG.static_files),
        )
        .mount(
            "/",
            routes![
                routes::gitlab::post_pipeline_update,
                routes::user::index,
                routes::user::robots,          // TODO: Add test
                routes::user::favicon,         // TODO: Add test
                routes::api::version,          // TODO: Add test
                routes::api::channel_version,  // TODO: Add test
                routes::api::download,         // TODO: Add test
                routes::api::channel_download, // TODO: Add test
            ],
        )
        .register(catchers![
            routes::catchers::not_found,
            routes::catchers::internal_error
        ])
}
