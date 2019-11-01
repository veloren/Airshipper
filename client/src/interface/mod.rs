use crate::config::ClientConfig;
use crate::Result;

mod cli;
mod gui;

pub fn process(mut config: ClientConfig) -> Result<()> {
    cli::run(&mut config)?;
    gui::run(&mut config);

    Ok(())
}
