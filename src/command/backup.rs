pub mod args;

use std::borrow::Cow;
use std::panic::resume_unwind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use mini_exercism::api;
use mini_exercism::api::v2::Solution;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{debug, enabled, event_enabled, info, instrument, trace, Level};

use crate::command::backup::args::BackupArgs;
use crate::credentials::get_api_credentials;
use crate::exercism::tracks::{get_joined_tracks, get_solutions};
use crate::exercism::{get_v1_client, get_v2_client};
use crate::fs::delete_directory_content;
use crate::reqwest::get_http_client;
use crate::task::wait_for_all;

#[instrument(skip_all)]
pub async fn backup_solutions(args: Cow<'static, BackupArgs>) -> crate::Result<()> {
    info!("Starting Exercism solutions backup");
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
    let v1_client = get_v1_client(&http_client, &credentials);
    let v2_client = get_v2_client(&http_client, &credentials);

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
                download_one_solution(v1_client, solution, output_path, &args, limiter).await
            });
        }

        wait_for_all(&mut downloads).await?;
    }

    info!("Exercism solutions backup complete");
    Ok(())
}

#[instrument(skip_all, ret(level = "trace"))]
async fn get_tracks_to_backup(
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

#[instrument(skip(tracks))]
async fn create_track_directories(
    output_path: &Path,
    tracks: &Vec<Cow<'_, str>>,
) -> crate::Result<()> {
    for track in tracks {
        let mut destination_path = output_path.to_path_buf();
        destination_path.push(track.as_ref());

        fs::create_dir_all(&destination_path)
            .await
            .with_context(|| format!("failed to create directory for track {}", track))?;
    }

    Ok(())
}

#[instrument(skip_all, ret(level = "trace"))]
async fn get_solutions_to_backup(
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

#[instrument(
    level = "debug",
    skip_all,
    fields(%solution.track.name, %solution.exercise.name)
)]
async fn download_one_solution(
    client: api::v1::Client,
    solution: Cow<'static, Solution>,
    output_path: Cow<'static, PathBuf>,
    args: &BackupArgs,
    limiter: Arc<Semaphore>,
) -> crate::Result<()> {
    if !args.dry_run {
        debug!("Starting solution backup");
    }
    trace!(?solution);

    let mut output_path = output_path;
    output_path.to_mut().push(&solution.track.name);
    output_path.to_mut().push(&solution.exercise.name);
    trace!(output_path = %output_path.display());

    if fs::metadata(&output_path.as_ref())
        .await
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
    {
        if args.force {
            trace!("Solution already exists on disk; cleaning up...");
            if !args.dry_run {
                delete_directory_content(&output_path)
                    .await
                    .with_context(|| {
                        format!("failed to clean up existing directory {}", output_path.display())
                    })?;
            }
        } else {
            trace!("Solution already exists on disk; skipping");
            return Ok(());
        }
    }

    if !args.dry_run {
        fs::create_dir_all(&output_path.as_ref())
            .await
            .with_context(|| {
                format!(
                    "failed to create destination directory for solution to {}/{}: {}",
                    solution.track.name,
                    solution.exercise.name,
                    output_path.display(),
                )
            })?;
    }

    let files = get_files_to_backup(&client, &solution).await?;
    if args.dry_run {
        debug!("Files to backup: {}", files.join(", "));
    }

    if !args.dry_run || enabled!(Level::TRACE) {
        let mut downloads = JoinSet::new();
        for file in files {
            let client = client.clone();
            let solution = solution.clone();
            let output_path = output_path.clone();
            let limiter = limiter.clone();
            let dry_run = args.dry_run;
            downloads.spawn(async move {
                let _permit = limiter
                    .acquire_owned()
                    .await
                    .expect("failed to acquire limiter semaphore");

                download_one_file(client, &solution, file, &output_path, dry_run).await
            });
        }

        wait_for_all(&mut downloads).await?;
    }

    Ok(())
}

#[instrument(
    skip_all,
    fields(%solution.track.name, %solution.exercise.name),
    ret(level = "trace")
)]
async fn get_files_to_backup(
    client: &api::v1::Client,
    solution: &Solution,
) -> crate::Result<Vec<String>> {
    Ok(client
        .get_solution(&solution.uuid)
        .await
        .with_context(|| {
            format!(
                "failed to get list of files for solution {}/{}",
                solution.track.name, solution.exercise.name,
            )
        })?
        .solution
        .files)
}

#[instrument(
    level = "trace",
    skip_all,
    fields(%solution.track.name, %solution.exercise.name, file)
)]
async fn download_one_file(
    client: api::v1::Client,
    solution: &Solution,
    file: String,
    destination_path: &Path,
    dry_run: bool,
) -> crate::Result<()> {
    let mut file_stream = client.get_file(&solution.uuid, &file).await;

    let mut destination_file_path = destination_path.to_path_buf();
    destination_file_path.extend(file.split('/'));
    trace!(destination_file_path = %destination_file_path.display());

    if !dry_run {
        if let Some(parent) = destination_file_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "failed to make sure parent of file {} exists",
                    destination_file_path.display()
                )
            })?;
        }

        let destination_file = fs::File::create(&destination_file_path)
            .await
            .with_context(|| {
                format!("failed to create local file {}", destination_file_path.display())
            })?;
        let mut destination_file = BufWriter::new(destination_file);

        while let Some(bytes) = file_stream.next().await {
            let bytes = bytes.with_context(|| {
                format!(
                    "failed to download file {} in solution to exercise {} of track {}",
                    file, solution.exercise.name, solution.track.name
                )
            })?;
            destination_file.write_all(&bytes).await.with_context(|| {
                format!("failed to write data to file {}", destination_file_path.display())
            })?;
        }

        destination_file.flush().await.with_context(|| {
            format!("failed to flush data to file {}", destination_file_path.display())
        })?;
    }

    Ok(())
}
