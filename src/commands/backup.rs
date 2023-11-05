pub mod args;

use std::borrow::Cow;
use std::panic::resume_unwind;
use std::path::Path;

use anyhow::Context;
use futures::StreamExt;
use log::{debug, info, trace, Level};
use mini_exercism::api;
use mini_exercism::api::v2::Solution;
use tokio::fs::{create_dir_all, metadata, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::task::JoinSet;

use crate::commands::backup::args::BackupArgs;
use crate::credentials::get_api_credentials;
use crate::exercism::tracks::{get_joined_tracks, get_solutions};
use crate::exercism::{get_v1_client, get_v2_client};
use crate::fs::delete_directory_content;
use crate::progress::ProgressBar;

pub async fn backup_solutions(args: &BackupArgs) -> crate::Result<()> {
    trace!("Starting solutions backup");
    debug!("BackupArgs: {:?}", args);

    trace!("Making sure output directory \"{}\" exists", args.path.display());
    create_dir_all(&args.path)
        .await
        .with_context(|| format!("failed to create output directory {}", args.path.display()))?;

    let output_path = args.path.canonicalize().with_context(|| {
        format!("failed to get absolute path for output directory {}", args.path.display())
    })?;
    info!("Starting solutions backup to {}", output_path.display());

    let credentials = get_api_credentials(args.token.as_ref())?;
    let v1_client = get_v1_client(&credentials)?;
    let v2_client = get_v2_client(&credentials)?;

    trace!("Getting list of tracks to backup");
    let tracks = get_tracks_to_backup(&v2_client, args).await?;
    debug!("{} tracks to backup", tracks.len());
    trace!("Tracks to backup: {}", tracks.join(", "));

    info!("Getting list of solutions to backup");
    let solutions = get_solutions_to_backup(&v2_client, &tracks, args).await?;
    debug!("{} solutions to backup", solutions.len());
    trace!("Solutions to backup: {}", {
        let mut solution_names = String::new();
        for solution in &solutions {
            if !solution_names.is_empty() {
                solution_names += ", ";
            }
            solution_names += &solution.track.name;
            solution_names.push('/');
            solution_names += &solution.exercise.name;
        }
        solution_names
    });

    let progress_bar = ProgressBar::for_log_level(Level::Info, solutions.len());
    for solution in &solutions {
        progress_bar.println(format!("{}/{}", solution.track.name, solution.exercise.name));
        download_one_solution(&v1_client, solution, &output_path, args)
            .await
            .with_context(|| {
                format!(
                    "failed to download solution to {}/{}",
                    solution.track.name, solution.exercise.name,
                )
            })?;
        progress_bar.inc(1);
    }

    Ok(())
}

async fn get_tracks_to_backup(
    client: &api::v2::Client,
    args: &BackupArgs,
) -> crate::Result<Vec<String>> {
    Ok(get_joined_tracks(client)
        .await
        .with_context(|| "failed to get list of tracks joined by user")?
        .into_iter()
        .filter(|track| args.track_matches(track))
        .collect())
}

async fn get_solutions_to_backup(
    client: &api::v2::Client,
    tracks: &Vec<String>,
    args: &BackupArgs,
) -> crate::Result<Vec<Solution>> {
    let mut solutions = Vec::new();
    {
        let progress_bar = ProgressBar::for_log_level(Level::Trace, tracks.len());

        let mut downloads = JoinSet::new();
        for track in tracks {
            let client = client.clone();
            let track = Cow::from(track.clone());
            downloads.spawn(async move {
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
                            .filter(|solution| args.solution_matches(solution)),
                    );
                    progress_bar.inc(1);
                },
                Err(err) => resume_unwind(err.into_panic()),
            }
        }
    }

    Ok(solutions)
}

async fn download_one_solution(
    client: &api::v1::Client,
    solution: &Solution,
    output_path: &Path,
    args: &BackupArgs,
) -> crate::Result<()> {
    let mut destination_path = output_path.to_path_buf();
    destination_path.push(&solution.track.name);
    destination_path.push(&solution.exercise.name);

    if metadata(&destination_path)
        .await
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
    {
        if args.force {
            trace!(
                "Solution to {}/{} already exists on disk; cleaning up...",
                solution.track.name,
                solution.exercise.name
            );
            delete_directory_content(&destination_path)
                .await
                .with_context(|| {
                    format!("failed to clean up existing directory {}", destination_path.display())
                })?;
        } else {
            trace!(
                "Solution to {}/{} already exists on disk; skipping",
                solution.track.name,
                solution.exercise.name
            );
            return Ok(());
        }
    }

    trace!(
        "Downloading solution to {}/{} to {}",
        solution.track.name,
        solution.exercise.name,
        destination_path.display()
    );
    create_dir_all(&destination_path).await.with_context(|| {
        format!(
            "failed to create destination directory for solution to {}/{}: {}",
            solution.track.name,
            solution.exercise.name,
            destination_path.display(),
        )
    })?;

    let files = get_files_to_backup(client, solution).await?;
    debug!("{} files to backup", files.len());
    trace!("Files to backup: {}", files.join(", "));

    let progress_bar = ProgressBar::for_log_level(Level::Trace, files.len());
    for file in &files {
        progress_bar.println(file);
        download_one_file(client, solution, file, &destination_path).await?;
        progress_bar.inc(1);
    }

    Ok(())
}

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

async fn download_one_file(
    client: &api::v1::Client,
    solution: &Solution,
    file: &str,
    destination_path: &Path,
) -> crate::Result<()> {
    let mut file_stream = client.get_file(&solution.uuid, file).await;

    let mut destination_file_path = destination_path.to_path_buf();
    destination_file_path.extend(file.split('/'));

    if let Some(parent) = destination_file_path.parent() {
        create_dir_all(parent).await.with_context(|| {
            format!("failed to make sure parent of file {} exists", destination_file_path.display())
        })?;
    }

    let destination_file = File::create(&destination_file_path)
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

    Ok(())
}
