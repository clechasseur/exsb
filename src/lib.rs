mod command;
mod error;

pub(crate) mod exercism;
pub(crate) mod fs;
pub(crate) mod reqwest;
pub(crate) mod task;

use std::str::FromStr;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
pub use error::Error;
pub use error::Result;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

use crate::command::Command;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn execute() -> Result<()> {
        let cli = Self::parse();

        let default_directive =
            Directive::from_str(&format!("{}={}", module_path!(), cli.verbose.log_level_filter()))
                .expect("default directive should be valid");
        let env_filter = EnvFilter::builder()
            .with_default_directive(default_directive)
            .from_env_lossy();
        tracing_subscriber::fmt().with_env_filter(env_filter).init();

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
