#![windows_subsystem = "windows"] // Removes terminal window

mod assets;
mod cli;
mod consts;
mod error;
mod gui;
mod io;
mod logger;
mod net;
mod profiles;
mod state;
#[cfg(windows)]
mod updater;

use crate::error::ClientError;

pub use io::*;
pub use net::*;

pub type Result<T> = std::result::Result<T, ClientError>;

#[async_std::main]
async fn main() {
    if let Err(e) = cli::process().await {
        log::error!("{}", e);
        log::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
