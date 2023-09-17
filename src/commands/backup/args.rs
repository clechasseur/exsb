use std::path::PathBuf;
use clap::{Args, ValueEnum};

#[derive(Debug, Args)]
pub struct BackupArgs {
    /// Path where to store the downloaded solutions
    pub path: PathBuf,

    /// API token to use to connect to the Exercism.org website; if unspecified,
    /// Exercism CLI token will be used if possible
    #[arg(long)]
    pub token: Option<String>,

    /// If specified, only solutions to exercises in the given track(s) will be downloaded; can be
    /// specified multiple times.
    #[arg(short, long)]
    pub track: Vec<String>,

    /// If specified, only solutions to the given exercise(s) will be downloaded; can be specified
    /// multiple times.
    #[arg(short, long)]
    pub exercise: Vec<String>,

    /// Filter out some solutions based on status
    #[arg(short, long, value_enum, default_value_t = SolutionStatus::Submitted)]
    pub status: SolutionStatus,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum SolutionStatus {
    /// At least one iteration has been submitted, but exercise has not been marked as complete
    Submitted,

    /// Exercise has been marked as complete
    Completed,

    /// Exercise has been marked as complete and a solution has been published
    Published,
}
