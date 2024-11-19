//! This module parses command line arguments and returns a parsed struct on which
//! the GUI/CLI can act upon.
use clap::{ArgAction::Count, crate_authors, crate_version, Parser, Subcommand};

/// Provides automatic updates for the voxel RPG Veloren. ( <https://veloren.net> )
#[derive(Parser, Debug, Default, Clone)]
#[command(name = "Airshipper", version = crate_version!(), author = crate_authors!())]
pub struct CmdLine {
    #[command(subcommand)]
    pub action: Option<Action>,
    /// Set the logging verbosity for Veloren (v = DEBUG, vv = TRACE)
    #[arg(short, long, action = Count, global = true)]
    pub verbose: u8,
    /// Set the logging verbosity for Airshipper (d = DEBUG, dd = TRACE)
    #[arg(short, long, action = Count, global = true)]
    pub debug: u8,
    /// Force a reset of all user data on startup
    #[arg(long, global = true)]
    pub force_reset: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Starts the game without updating.
    Start,
    /// Only updates the game.
    Update,
    /// Update and start the game.
    Run,
    /// Use the CLI to configure profiles.
    Config,
    /// Update the Launcher if possible.
    #[cfg(windows)]
    Upgrade,
}

impl CmdLine {
    /// Parses command line for arguments and returns itself
    pub(crate) fn new() -> Self {
        CmdLine::parse()
    }
}
