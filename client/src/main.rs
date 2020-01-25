mod cli;
mod error;
/// Deals with all filesystem specific details
mod filesystem;
mod gui;
mod logger;
/// Takes care of all network operations
mod network;
mod profiles;
/// State which is used by the command line and GUI and also gets saved to disk
mod state;

use crate::error::ClientError;

pub type Result<T> = std::result::Result<T, ClientError>;

#[async_std::main]
async fn main() {
    if let Err(e) = cli::process().await {
        log::error!("{}", e);
        log::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
