#![windows_subsystem = "windows"] // Removes terminal window

mod assets;
mod cli;
mod error;
mod filesystem;
mod gui;
mod logger;
mod network;
mod profiles;
mod state;
#[cfg(windows)]
mod updater;

use crate::error::ClientError;

pub type Result<T> = std::result::Result<T, ClientError>;

#[async_std::main]
async fn main() {
    error::setup_panic_hook();
    if let Err(e) = cli::process().await {
        log::error!("{}", e);
        log::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
