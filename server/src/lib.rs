#![feature(proc_macro_hygiene)]
#![allow(clippy::unit_arg)]
use rocket::*;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod config;
mod db;
mod error;
mod fairings;
mod guards;
mod models;
mod prune;
mod routes;
mod webhook;

use crate::error::ServerError;
use config::ServerConfig;

pub type Result<T> = std::result::Result<T, ServerError>;
pub use db::{DbConnection, S3Connection};

lazy_static::lazy_static! {
    /// Contains all configuration needed.
    pub static ref CONFIG: ServerConfig = ServerConfig::load();
}

pub fn rocket() -> rocket::Rocket {
    // Base of the config and attach everything else
    CONFIG
        .rocket()
        .attach(DbConnection::fairing())
        .attach(fairings::db::DbInit)
        .mount("/", routes![
            routes::gitlab::post_pipeline_update,
            routes::user::index,
            routes::user::robots,
            routes::user::favicon,
            routes::api::version,
            routes::api::channel_version,
            routes::api::download,
            routes::api::channel_download,
        ])
        .register(catchers![routes::catchers::not_found])
}
