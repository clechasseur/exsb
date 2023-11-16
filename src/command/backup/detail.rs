pub mod exercism;
pub mod fs;

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use mini_exercism::api;
use mini_exercism::api::v2::Solution;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{debug, enabled, instrument, trace, Level};

use crate::command::backup::args::BackupArgs;
use crate::command::backup::detail::exercism::get_files_to_backup;
use crate::fs::delete_directory_content;
use crate::task::wait_for_all;

#[instrument(
    level = "debug",
    skip_all,
    fields(%solution.track.name, %solution.exercise.name)
)]
pub async fn download_one_solution(
    client: api::v1::Client,
    solution: Cow<'static, Solution>,
    output_path: Cow<'static, PathBuf>,
    args: &BackupArgs,
    limiter: Arc<Semaphore>,
) -> crate::Result<bool> {
    if !args.dry_run {
        debug!("Starting solution backup");
    }
    trace!(?solution);

    let mut output_path = output_path;
    output_path.to_mut().push(&solution.track.name);
    output_path.to_mut().push(&solution.exercise.name);
    trace!(output_path = %output_path.display());

    if tokio::fs::metadata(&output_path.as_ref())
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
            return Ok(false);
        }
    }

    if !args.dry_run {
        tokio::fs::create_dir_all(&output_path.as_ref())
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

    Ok(true)
}

#[instrument(
    level = "trace",
    skip_all,
    fields(%solution.track.name, %solution.exercise.name, file)
)]
pub async fn download_one_file(
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
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "failed to make sure parent of file {} exists",
                    destination_file_path.display()
                )
            })?;
        }

        let destination_file = tokio::fs::File::create(&destination_file_path)
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
