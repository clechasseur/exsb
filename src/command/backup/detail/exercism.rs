//! Implementation details of the [`Backup`](crate::command::Command::Backup) command pertaining
//! to the Exercism API.

use std::borrow::Cow;
use std::panic::resume_unwind;
use std::sync::Arc;

use anyhow::Context;
use mini_exercism::api;
use mini_exercism::api::v2::solution::Solution;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::instrument;

use crate::command::backup::args::BackupArgs;
use crate::exercism::solutions::get_solution_files;
use crate::exercism::tracks::{get_joined_tracks, get_solutions};

/// Gets the list of tracks to backup, depending on [command-line arguments](BackupArgs).
#[instrument(skip_all, ret(level = "trace"))]
pub async fn get_tracks_to_backup(
    client: &api::v2::Client,
    args: &BackupArgs,
) -> crate::Result<Vec<Cow<'static, str>>> {
    Ok(get_joined_tracks(client)
        .await
        .with_context(|| "failed to get list of tracks joined by user")?
        .into_iter()
        .filter(|track| args.track_matches(track))
        .map(From::from)
        .collect())
}

/// Gets the list of solutions to backup for the given `track`, depending on [command-line arguments](BackupArgs).
#[instrument(skip_all, ret(level = "trace"))]
pub async fn get_solutions_to_backup(
    client: &api::v2::Client,
    tracks: &Vec<Cow<'static, str>>,
    args: &BackupArgs,
    limiter: &Arc<Semaphore>,
) -> crate::Result<Vec<Cow<'static, Solution>>> {
    let mut solutions = Vec::new();
    {
        let mut downloads = JoinSet::new();
        for track in tracks {
            let client = client.clone();
            let track = track.clone();
            let limiter = limiter.clone();
            downloads.spawn(async move {
                let _permit = limiter
                    .acquire_owned()
                    .await
                    .expect("failed to acquire limiter semaphore");

                let solution_track = track.clone();
                (track, get_solutions(&client, solution_track).await)
            });
        }

        while let Some(join_result) = downloads.join_next().await {
            match join_result {
                Ok((track, track_solutions)) => {
                    solutions.extend(
                        track_solutions
                            .with_context(|| {
                                format!("failed to download solutions for track {}", track)
                            })?
                            .into_iter()
                            .filter(|solution| args.solution_matches(solution))
                            .map(Cow::<'static, _>::Owned),
                    );
                },
                Err(err) => resume_unwind(err.into_panic()),
            }
        }
    }

    Ok(solutions)
}

/// Gets the list of files to backup for the given [`Solution`].
#[instrument(
    skip_all,
    fields(%solution.track.name, %solution.exercise.name),
    ret(level = "trace")
)]
pub async fn get_files_to_backup(
    client: &api::v1::Client,
    solution: &Solution,
) -> crate::Result<Vec<String>> {
    get_solution_files(client, &solution.uuid)
        .await
        .with_context(|| {
            format!(
                "failed to get list of files for solution {}/{}",
                solution.track.name, solution.exercise.name
            )
        })
}
