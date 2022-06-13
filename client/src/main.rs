#![feature(async_closure)]
#![feature(let_chains)]
mod assets;
mod cli;
mod consts;
mod error;
mod gui;
mod io;
mod logger;
mod net;
#[cfg(unix)]
mod nix;
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
        tracing::error!("{}", e);
        tracing::info!("Press enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
