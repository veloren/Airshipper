// https://github.com/rust-lang/rust/pull/40870
#![windows_subsystem = "windows"]

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
    // Required on windows due to stdin/stdout not being attached by default
    // to the parent process if `#![windows_subsystem = "windows"]` is used.
    #[cfg(windows)]
    unsafe {
        let _ = winapi::um::wincon::AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS);
    }
    
    if let Err(e) = cli::process().await {
        log::error!("{}", e);
        log::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
