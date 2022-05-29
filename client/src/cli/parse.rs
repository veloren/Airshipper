//! This module parses command line arguments and returns a parsed struct on which
//! the GUI/CLI can act upon.
use clap::{crate_authors, crate_version, Parser, Subcommand};

/// Provides automatic updates for the voxel RPG Veloren. ( <https://veloren.net> )
#[derive(Parser, Debug, Default, Clone)]
#[clap(name = "Airshipper", version = crate_version!(), author = crate_authors!())]
pub struct CmdLine {
    #[clap(subcommand)]
    pub action: Option<Action>,
    /// Set the logging verbosity for Veloren (v = DEBUG, vv = TRACE)
    #[clap(short, long, parse(from_occurrences), global = true)]
    pub verbose: i32,
    /// Set the logging verbosity for Airshipper (d = DEBUG, dd = TRACE)
    #[clap(short, long, parse(from_occurrences), global = true)]
    pub debug: i32,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Starts the game without updating.
    Start,
    /// Only updates the game.
    Update,
    /// Update and start the game.
    Run,
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
