mod error;
mod gui;
mod logger;
mod network;
mod profiles;
mod saved_state;

use crate::error::ClientError;

#[cfg(windows)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen.exe";
#[cfg(unix)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen";

#[cfg(windows)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli.exe";
#[cfg(unix)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli";

// TODO: * add tests

pub type Result<T> = std::result::Result<T, ClientError>;

fn main() {
    let _ = logger::log(log::LevelFilter::Info);
    gui::run();
}
