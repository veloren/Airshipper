mod cli;
mod error;
mod filesystem;
#[cfg(feature = "gui")]
mod gui;
mod logger;
mod network;
mod profiles;
mod state;
#[cfg(windows)]
mod updater;

use crate::error::ClientError;

pub type Result<T> = std::result::Result<T, ClientError>;

pub fn start() {
    async_std::task::block_on(async {
        error::setup_panic_hook();
        if let Err(e) = cli::process().await {
            log::error!("{}", e);
            log::info!("Press enter to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    });
}
