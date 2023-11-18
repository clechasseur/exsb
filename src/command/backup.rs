//! Implementation of the [`Backup`](crate::command::Command::Backup) command.

pub mod args;
mod detail;

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{enabled, event_enabled, info, instrument, trace, Level};

use crate::command::backup::args::BackupArgs;
use crate::command::backup::detail::download_one_solution;
use crate::command::backup::detail::exercism::{get_solutions_to_backup, get_tracks_to_backup};
use crate::command::backup::detail::fs::create_track_directories;
use crate::exercism::credentials::get_api_credentials;
use crate::exercism::{get_v1_client, get_v2_client};
use crate::reqwest::get_http_client;
use crate::task::wait_for_all;

/// Downloads all solutions for backup.
///
/// Uses the provided [`args`](BackupArgs) to determine where to store the backed up
/// solutions, which solutions to download and whether to overwrite existing ones.
#[instrument(skip_all)]
pub async fn backup_solutions(args: Cow<'static, BackupArgs>) -> crate::Result<()> {
    info!("Starting Exercism solutions backup to {}", args.path.display());
    trace!(?args);

    if !args.dry_run {
        fs::create_dir_all(&args.path).await.with_context(|| {
            format!("failed to create output directory {}", args.path.display())
        })?;
    }

    let output_path =
        Cow::<'static, PathBuf>::Owned(args.path.canonicalize().with_context(|| {
            format!("failed to get absolute path for output directory {}", args.path.display())
        })?);
    trace!(output_path = %output_path.display());

    let credentials = get_api_credentials(args.token.as_ref())?;
    let http_client = get_http_client()?;
    let v1_client = get_v1_client(&http_client, &credentials, None);
    let v2_client = get_v2_client(&http_client, &credentials, None);

    let tracks = get_tracks_to_backup(&v2_client, &args).await?;
    info!("Number of tracks to scan: {}", tracks.len());

    if !args.dry_run {
        // Create track directories right away so that concurrent tasks don't end up trying
        // to create a directory multiple times.
        create_track_directories(&output_path, &tracks).await?;
    }

    let limiter = Arc::new(Semaphore::new(args.max_downloads));

    let solutions = get_solutions_to_backup(&v2_client, &tracks, &args, &limiter).await?;
    info!("Number of solutions to backup: {}", solutions.len());

    if args.dry_run && event_enabled!(Level::INFO) {
        let solutions_list = solutions
            .iter()
            .map(|solution| format!("{}/{}", solution.track.name, solution.exercise.name))
            .collect::<Vec<_>>()
            .join(", ");
        info!("Solutions to backup: {}", solutions_list);
    }

    if !args.dry_run || enabled!(Level::DEBUG) {
        let mut downloads = JoinSet::new();
        for solution in &solutions {
            let v1_client = v1_client.clone();
            let solution = solution.clone();
            let output_path = output_path.clone();
            let args = args.clone();
            let limiter = limiter.clone();
            downloads.spawn(async move {
                download_one_solution(v1_client, solution.clone(), output_path, &args, limiter)
                    .await
                    .map(|downloaded| {
                        if downloaded {
                            info!(
                                "Solution to {}/{} downloaded",
                                solution.track.name, solution.exercise.name
                            );
                        } else {
                            info!(
                                "Solution to {}/{} already exists; skipped.",
                                solution.track.name, solution.exercise.name
                            );
                        }
                    })
            });
        }

        wait_for_all(&mut downloads).await?;
    }

    info!("Exercism solutions backup complete");
    Ok(())
}
