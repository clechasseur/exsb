mod commands;
mod error;

pub(crate) mod credentials;
pub(crate) mod exercism;
pub(crate) mod fs;
pub(crate) mod reqwest;
pub(crate) mod task;
pub(crate) mod tracing;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
pub use error::Error;
pub use error::Result;

use crate::commands::Commands;
use crate::tracing::log_level_to_tracing_level;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    verbose: Verbosity<WarnLevel>,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub async fn execute() -> Result<()> {
        let cli = Self::parse();

        tracing_subscriber::fmt()
            .with_max_level(cli.verbose.log_level().map(log_level_to_tracing_level))
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
