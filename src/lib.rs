mod commands;
mod error;

pub(crate) mod credentials;
pub(crate) mod exercism;
pub(crate) mod reqwest;

use clap::Parser;
use crate::commands::Commands;

pub use error::Error;
pub use error::Result;

/// Backup your Exercism.org solutions
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn execute(&self) -> Result<()> {
        self.command.execute()
    }
}
