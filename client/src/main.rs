mod config;
mod error;
mod interface;
mod logger;
mod models;
mod server;

use crate::config::ClientConfig;
use crate::error::ClientError;

// TODO: * add tests

pub type Result<T> = std::result::Result<T, ClientError>;

fn main() {
    let config = ClientConfig::load();
    if let Err(e) = interface::process(config) {
        log::error!("{}", e);
        log::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}