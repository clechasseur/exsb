pub mod args;

use std::fs;
use std::path::Path;

use anyhow::Context;
use futures::StreamExt;
use mini_exercism::api;
use mini_exercism::api::v2::Solution;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::commands::backup::args::{BackupArgs, SolutionStatus};
use crate::credentials::get_api_credentials;
use crate::exercism::tracks::{get_joined_tracks, get_solutions};
use crate::exercism::{get_v1_client, get_v2_client};

pub async fn backup_solutions(args: &BackupArgs) -> crate::Result<()> {
    fs::create_dir_all(&args.path)
        .with_context(|| format!("Failed to create output directory {}", args.path.display()))?;
    let output_path = args.path.canonicalize().with_context(|| {
        format!("Failed to get absolute path for output directory {}", args.path.display())
    })?;

    let credentials = get_api_credentials(args.token.as_ref())?;
    let v1_client = get_v1_client(&credentials)?;
    let v2_client = get_v2_client(&credentials)?;

    let tracks = get_tracks_to_backup(&v2_client, args).await?;
    let solutions = get_solutions_to_backup(&v2_client, &tracks, args).await?;
    for solution in &solutions {
        download_one_solution(&v1_client, solution, &output_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to download solution to exercise {} in track {}",
                    solution.exercise.name, solution.track.name
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
        .with_context(|| "Failed to get list of tracks joined by user")?
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
                    format!("Failed to get solutions submitted by user for track {}", track)
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
) -> crate::Result<()> {
    let mut destination_path = output_path.to_path_buf();
    destination_path.push(&solution.track.name);
    destination_path.push(&solution.exercise.name);
    fs::create_dir_all(&destination_path).with_context(|| {
        format!(
            "Failed to create destination directory for solution to exercise {} in track {}: {}",
            solution.exercise.name,
            solution.track.name,
            destination_path.display()
        )
    })?;

    let files = get_files_to_backup(client, solution).await?;
    for file in &files {
        download_one_file(client, solution, file, &destination_path).await?;
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
                "Failed to get list of files for solution to exercise {} in track {}",
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
    let mut file_stream = client.get_file(&solution.uuid, file).await;

    let mut destination_file_path = destination_path.to_path_buf();
    destination_file_path.extend(file.split('/'));
    if let Some(parent) = destination_file_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to make sure parent of file {} exists", destination_file_path.display())
        })?;
    }

    let destination_file = File::create(&destination_file_path)
        .await
        .with_context(|| {
            format!("Failed to create local file {}", destination_file_path.display())
        })?;
    let mut destination_file = BufWriter::new(destination_file);

    while let Some(bytes) = file_stream.next().await {
        let bytes = bytes.with_context(|| {
            format!(
                "Failed to download file {} in solution to exercise {} of track {}",
                file, solution.exercise.name, solution.track.name
            )
        })?;
        destination_file.write_all(&bytes).await.with_context(|| {
            format!("Failed to write data to file {}", destination_file_path.display())
        })?;
    }

    destination_file.flush().await.with_context(|| {
        format!("Failed to flush data to file {}", destination_file_path.display())
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
