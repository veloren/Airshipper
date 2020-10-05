mod assets;
mod cli;
mod consts;
mod error;
mod gui;
mod io;
mod logger;
mod net;
mod profiles;
#[cfg(windows)]
mod windows;

use crate::error::ClientError;

pub use io::*;
pub use net::*;

pub type Result<T> = std::result::Result<T, ClientError>;

fn main() {
    error::panic_hook();

    if let Err(e) = cli::process() {
        log::error!("{}", e);
        log::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
