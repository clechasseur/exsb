mod commands;
mod error;

pub(crate) mod credentials;
pub(crate) mod exercism;
pub(crate) mod reqwest;

use clap::Parser;
pub use error::Error;
pub use error::Result;

use crate::commands::Commands;

/// Backup your Exercism.org solutions
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub async fn execute() -> Result<()> {
        Self::parse().command.execute().await
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn test_cli() {
        Cli::command().debug_assert();
    }
}
