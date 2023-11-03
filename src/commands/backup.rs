pub mod args;

use std::path::Path;

use anyhow::Context;
use futures::StreamExt;
use indicatif::ProgressBar;
use log::{debug, info, log_enabled, trace, Level};
use mini_exercism::api;
use mini_exercism::api::v2::Solution;
use tokio::fs::{create_dir_all, metadata, read_dir, remove_dir_all, remove_file, File};
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::commands::backup::args::{BackupArgs, SolutionStatus};
use crate::credentials::get_api_credentials;
use crate::exercism::tracks::{get_joined_tracks, get_solutions};
use crate::exercism::{get_v1_client, get_v2_client};

pub async fn backup_solutions(args: &BackupArgs) -> crate::Result<()> {
    info!("Starting solutions backup");
    debug!("BackupArgs: {:?}", args);

    info!("Making sure output directory \"{}\" exists", args.path.display());
    create_dir_all(&args.path)
        .await
        .with_context(|| format!("failed to create output directory {}", args.path.display()))?;

    let output_path = args.path.canonicalize().with_context(|| {
        format!("failed to get absolute path for output directory {}", args.path.display())
    })?;
    debug!("Absolute output path: {}", output_path.display());

    let credentials = get_api_credentials(args.token.as_ref())?;
    let v1_client = get_v1_client(&credentials)?;
    let v2_client = get_v2_client(&credentials)?;

    info!("Getting list of tracks to backup");
    let tracks = get_tracks_to_backup(&v2_client, args).await?;
    debug!("{} tracks to backup", tracks.len());

    info!("Getting list of solutions to backup");
    let solutions = get_solutions_to_backup(&v2_client, &tracks, args).await?;
    debug!("{} solutions to backup", solutions.len());

    for solution in &solutions {
        download_one_solution(&v1_client, solution, &output_path, args)
            .await
            .with_context(|| {
                format!(
                    "failed to download solution to {}/{}",
                    solution.track.name, solution.exercise.name,
                )
            })?;
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
        .filter(|track| should_download_track(track, args))
        .collect())
}

async fn get_solutions_to_backup(
    client: &api::v2::Client,
    tracks: &Vec<String>,
    args: &BackupArgs,
) -> crate::Result<Vec<Solution>> {
    let mut solutions = Vec::new();
    for track in tracks {
        solutions.extend(
            get_solutions(client, track)
                .await
                .with_context(|| {
                    format!("failed to get solutions submitted by user for track {}", track)
                })?
                .into_iter()
                .filter(|solution| should_download_solution(solution, args)),
        );
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
            info!(
                "Solution to {}/{} already exists on disk; cleaning up...",
                solution.track.name, solution.exercise.name
            );
            delete_directory_content(&destination_path)
                .await
                .with_context(|| {
                    format!("failed to clean up existing directory {}", destination_path.display())
                })?;
        } else {
            info!(
                "Solution to {}/{} already exists on disk; skipping",
                solution.track.name, solution.exercise.name
            );
            return Ok(());
        }
    }

    info!(
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

    let progress_bar = log_enabled!(Level::Info).then(|| ProgressBar::new(files.len() as u64));
    for file in &files {
        if let Some(progress_bar) = &progress_bar {
            progress_bar.println(file);
            progress_bar.inc(1);
        }
        download_one_file(client, solution, file, &destination_path).await?;
    }
    if let Some(progress_bar) = &progress_bar {
        progress_bar.finish_and_clear();
    }

    Ok(())
}

async fn delete_directory_content(directory_path: &Path) -> crate::Result<()> {
    trace!("Removing content of directory {}", directory_path.display());
    let mut destination_entries = read_dir(directory_path).await.with_context(|| {
        format!("failed to get content of directory {}", directory_path.display())
    })?;

    loop {
        let entry = destination_entries.next_entry().await.with_context(|| {
            format!("failed to get next entry of directory {}", directory_path.display())
        })?;

        if let Some(entry) = entry {
            let entry_type = entry.file_type().await.with_context(|| {
                format!("failed to fetch file type for entry {}", entry.path().display())
            })?;

            if entry_type.is_dir() {
                trace!("Removing directory {:?}", entry.file_name());
                remove_dir_all(entry.path()).await.with_context(|| {
                    format!("failed to remove directory {}", entry.path().display())
                })?;
            } else {
                trace!("Removing file {:?}", entry.file_name());
                remove_file(entry.path())
                    .await
                    .with_context(|| format!("failed to remove file {}", entry.path().display()))?;
            }
        } else {
            break;
        }
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
                "failed to get list of files for solution to exercise {} in track {}",
                solution.exercise.name, solution.track.name
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
    trace!(
        "Downloading file {} for solution to {}/{}",
        file,
        solution.track.name,
        solution.exercise.name
    );

    let mut file_stream = client.get_file(&solution.uuid, file).await;

    let mut destination_file_path = destination_path.to_path_buf();
    destination_file_path.extend(file.split('/'));

    if let Some(parent) = destination_file_path.parent() {
        trace!("Making sure destination directory {} exists", parent.display());
        create_dir_all(parent).await.with_context(|| {
            format!("failed to make sure parent of file {} exists", destination_file_path.display())
        })?;
    }

    trace!("Creating destination file \"{}\"", destination_file_path.display());
    let destination_file = File::create(&destination_file_path)
        .await
        .with_context(|| {
            format!("failed to create local file {}", destination_file_path.display())
        })?;
    let mut destination_file = BufWriter::new(destination_file);

    trace!("Transferring file content for \"{}\"", destination_file_path.display());
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

fn should_download_track(track: &str, args: &BackupArgs) -> bool {
    args.track.is_empty() || args.track.iter().any(|t| t == track)
}

fn should_download_solution(solution: &Solution, args: &BackupArgs) -> bool {
    let solution_status: Option<SolutionStatus> = solution.status.try_into().ok();

    (args.status == SolutionStatus::Submitted
        || solution_status.map_or(false, |st| st >= args.status))
        && (args.exercise.is_empty() || args.exercise.iter().any(|e| e == &solution.exercise.name))
}
