mod commands;
mod error;

pub(crate) mod credentials;
pub(crate) mod exercism;
pub(crate) mod reqwest;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
pub use error::Error;
pub use error::Result;

use crate::commands::Commands;

/// Backup your Exercism.org solutions
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub async fn execute() -> Result<()> {
        let cli = Self::parse();

        env_logger::builder()
            .filter_level(cli.verbose.log_level_filter())
            .init();

        cli.command.execute().await
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
