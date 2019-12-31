use crate::config::ClientConfig;
use crate::Result;

mod cli;

pub use cli::downloadbar;

pub fn process(mut config: ClientConfig) -> Result<()> {
    cli::run(&mut config)?;

    Ok(())
}
