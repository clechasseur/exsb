use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

fn default_backup_path() -> PathBuf {
    ".".into()
}

/// Backup your Exercism.org solutions
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Download Exercism.org solutions for backup
    ///
    /// By default, this command will attempt to download backups of all solutions to exercises
    /// submitted to the Exercism.org website, for all language tracks, and will store them in
    /// the current directory. See options for ways to configure the output directory, which solutions
    /// to download, etc.
    ///
    /// If an exercise has had multiple iterations submitted, the latest iteration is always downloaded.
    ///
    /// To download solutions, an Exercism API token is needed. If not specified via the --token option,
    /// by default, the API token configured for the local installation of the Exercism CLI application
    /// will be used. The command does not require the Exercism CLI to work, but if it's not installed,
    /// then the API token will have to be specified (see --token).
    Backup(BackupArgs),
}

#[derive(Debug, Args)]
struct BackupArgs {
    /// Path where to store the downloaded solutions
    #[arg(default_value = default_backup_path().into_os_string())]
    path: PathBuf,

    /// API token to use to connect to the Exercism.org website; if unspecified,
    /// Exercism CLI token will be used if possible
    #[arg(long)]
    token: Option<String>,

    /// If specified, only solutions to exercises in the given track(s) will be downloaded; can be
    /// specified multiple times.
    #[arg(short, long)]
    track: Vec<String>,

    /// If specified, only solutions to the given exercise(s) will be downloaded; can be specified
    /// multiple times.
    #[arg(short, long)]
    exercise: Vec<String>,

    /// Filter out some solutions based on status
    #[arg(short, long, value_enum, default_value_t = SolutionStatus::Submitted)]
    status: SolutionStatus,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum SolutionStatus {
    /// At least one iteration has been submitted, but exercise has not been marked as complete
    Submitted,

    /// Exercise has been marked as complete
    Completed,

    /// Exercise has been marked as complete and a solution has been published
    Published,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Backup(args) => {
            println!("Args: {:?}", args);
        },
    }
}
