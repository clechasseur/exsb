mod backup;

use clap::Subcommand;
use crate::commands::backup::args::BackupArgs;
use crate::commands::backup::backup_solutions;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download Exercism.org solutions for backup
    ///
    /// By default, this command will attempt to download backups of all solutions to exercises
    /// submitted to the Exercism.org website, for all language tracks, and will store them in
    /// the specified directory. See options for ways to filter solutions/exercises to download, etc.
    ///
    /// If an exercise has had multiple iterations submitted, the latest iteration is always downloaded.
    ///
    /// To download solutions, an Exercism API token is needed. If not specified via the --token option,
    /// by default, the API token configured for the local installation of the Exercism CLI application
    /// will be used. The command does not require the Exercism CLI to work, but if it's not installed,
    /// then the API token will have to be specified (see --token).
    Backup(BackupArgs),
}

impl Commands {
    pub fn execute(&self) -> crate::Result<()> {
        match &self {
            Commands::Backup(args) => backup_solutions(args),
        }
    }
}
