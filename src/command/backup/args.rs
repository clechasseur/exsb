use std::path::PathBuf;

use clap::{Args, ValueEnum};
use mini_exercism::api;
use mini_exercism::api::v2::Solution;

#[derive(Debug, Clone, Args)]
pub struct BackupArgs {
    /// Path where to store the downloaded solutions
    pub path: PathBuf,

    /// Exercism.org API token; if unspecified, CLI token will be used instead
    #[arg(long)]
    pub token: Option<String>,

    /// Only download solutions in the given track(s) (can be used multiple times)
    #[arg(short, long)]
    pub track: Vec<String>,

    /// Only download solutions for the given exercise(s) (can be used multiple times)
    #[arg(short, long)]
    pub exercise: Vec<String>,

    /// Only download solutions with the given status (or greater)
    #[arg(short, long, value_enum, default_value_t = SolutionStatus::Submitted)]
    pub status: SolutionStatus,

    /// Overwrite exercises that have already been downloaded
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// Determine what solutions to backup without downloading them
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Maximum number of concurrent downloads
    #[arg(short, long, default_value_t = 4)]
    pub max_downloads: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SolutionStatus {
    /// At least one iteration has been submitted, but exercise has not been marked as complete
    Submitted,

    /// Exercise has been marked as complete
    Completed,

    /// Exercise has been marked as complete and a solution has been published
    Published,
}

impl TryFrom<api::v2::SolutionStatus> for SolutionStatus {
    type Error = ();

    fn try_from(value: api::v2::SolutionStatus) -> Result<Self, Self::Error> {
        match value {
            api::v2::SolutionStatus::Iterated => Ok(SolutionStatus::Submitted),
            api::v2::SolutionStatus::Completed => Ok(SolutionStatus::Completed),
            api::v2::SolutionStatus::Published => Ok(SolutionStatus::Published),
            _ => Err(()),
        }
    }
}

impl BackupArgs {
    pub fn track_matches(&self, track: &str) -> bool {
        self.track.is_empty() || self.track.iter().any(|t| t == track)
    }

    pub fn solution_matches(&self, solution: &Solution) -> bool {
        self.solution_status_matches(solution.status.try_into().ok())
            && self.exercise_matches(&solution.exercise.name)
    }

    fn solution_status_matches(&self, solution_status: Option<SolutionStatus>) -> bool {
        self.status == SolutionStatus::Submitted
            || solution_status.map_or(false, |st| st >= self.status)
    }

    fn exercise_matches(&self, exercise_name: &str) -> bool {
        self.exercise.is_empty() || self.exercise.iter().any(|e| e == exercise_name)
    }
}