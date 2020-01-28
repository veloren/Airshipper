mod cli;
mod error;
mod filesystem;
mod gui;
mod logger;
mod network;
mod profiles;
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
