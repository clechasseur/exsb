//! Definition of the [`Backup`](crate::command::Command::Backup) command.

pub mod args;
#[macro_use]
mod detail;

use std::collections::HashSet;
use std::panic::resume_unwind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use mini_exercism::api;
use mini_exercism::api::v2::solution::Solution;
use mini_exercism::api::v2::solutions;
use mini_exercism::cli::get_cli_credentials;
use mini_exercism::core::Credentials;
use tokio::{fs, spawn};
use tracing::{event_enabled, info, instrument, trace, Level};

use crate::command::backup::args::BackupArgs;
use crate::download_limiter::DownloadLimiter;
use crate::task_pool::TaskPool;
use crate::Result;

/// Command wrapper used for the [`Backup`](crate::command::Command::Backup) command.
///
/// # Notes
///
/// The [`new`](BackupCommand::new) method returns a [`BackupCommand`] wrapped in an [`Arc`], because it is
/// needed to adequately create asynchronous task. To use:
///
/// ```no_run
/// # use exsb::command::backup::args::BackupArgs;
/// use exsb::command::backup::BackupCommand;
///
/// # async fn perform_backup(args: BackupArgs) -> exsb::Result<()> {
/// let backup_command = BackupCommand::new(args, None)?;
/// BackupCommand::execute(backup_command).await
/// # }
/// ```
#[derive(Debug)]
pub struct BackupCommand {
    args: BackupArgs,
    _v1_client: api::v1::Client,
    v2_client: api::v2::Client,
    limiter: DownloadLimiter,
}

impl BackupCommand {
    /// Creates a new [`BackupCommand`] using the provided [`args`](BackupArgs).
    ///
    /// The `api_base_url` parameter should only be set to test using a different Exercism local endpoint.
    pub fn new(args: BackupArgs, api_base_url: Option<&str>) -> Result<Arc<Self>> {
        let http_client = reqwest::Client::builder()
            .build()
            .with_context(|| "failed to create HTTP client")?;
        let credentials = args
            .token
            .as_ref()
            .map(|token| Ok(Credentials::from_api_token(token)))
            .unwrap_or_else(|| {
                get_cli_credentials().with_context(|| "failed to get Exercism CLI credentials")
            })?;

        let v1_client = build_client!(api::v1::Client, http_client, credentials, api_base_url);
        let v2_client = build_client!(api::v2::Client, http_client, credentials, api_base_url);
        let limiter = DownloadLimiter::new(args.max_downloads);

        Ok(Arc::new(Self { args, _v1_client: v1_client, v2_client, limiter }))
    }

    /// Execute the backup operation.
    ///
    /// See [struct description](BackupCommand) for details on how to call this method.
    #[instrument(skip_all)]
    pub async fn execute(this: Arc<Self>) -> Result<()> {
        info!("Starting Exercism solutions backup to {}", this.args.path.display());
        trace!(?this.args);

        this.create_output_directory(&this.args.path).await?;

        let output_path = this.args.path.canonicalize().with_context(|| {
            format!("failed to get absolute path for output directory {}", this.args.path.display())
        })?;
        trace!(output_path = %output_path.display());

        match spawn(Self::backup_solutions(this.clone(), output_path)).await {
            Ok(Ok(())) => {
                info!("Exercism solutions backup complete");
                Ok(())
            },
            Ok(Err(task_error)) => return Err(task_error),
            Err(join_error) => resume_unwind(join_error.into_panic()),
        }

        // let tracks = get_tracks_to_backup(&v2_client, &args).await?;
        // info!("Number of tracks to scan: {}", tracks.len());
        //
        // if !args.dry_run {
        //     // Create track directories right away so that concurrent tasks don't end up trying
        //     // to create a directory multiple times.
        //     create_track_directories(&output_path, &tracks).await?;
        // }
        //
        // let limiter = Arc::new(Semaphore::new(args.max_downloads));
        //
        // let solutions = get_solutions_to_backup(&v2_client, &tracks, &args, &limiter).await?;
        // info!("Number of solutions to backup: {}", solutions.len());
        //
        // if args.dry_run && event_enabled!(Level::INFO) {
        //     let solutions_list = solutions
        //         .iter()
        //         .map(|solution| format!("{}/{}", solution.track.name, solution.exercise.name))
        //         .collect::<Vec<_>>()
        //         .join(", ");
        //     info!("Solutions to backup: {}", solutions_list);
        // }
        //
        // if !args.dry_run || enabled!(Level::DEBUG) {
        //     let mut downloads = JoinSet::new();
        //     for solution in &solutions {
        //         let v1_client = v1_client.clone();
        //         let solution = solution.clone();
        //         let output_path = output_path.clone();
        //         let args = args.clone();
        //         let limiter = limiter.clone();
        //         downloads.spawn(async move {
        //             download_one_solution(v1_client, solution.clone(), output_path, &args, limiter)
        //                 .await
        //                 .map(|downloaded| {
        //                     if downloaded {
        //                         info!(
        //                         "Solution to {}/{} downloaded",
        //                         solution.track.name, solution.exercise.name
        //                     );
        //                     } else {
        //                         info!(
        //                         "Solution to {}/{} already exists; skipped.",
        //                         solution.track.name, solution.exercise.name
        //                     );
        //                     }
        //                 })
        //         });
        //     }
        //
        //     wait_for_all(&mut downloads).await?;
        // }
        //
        // info!("Exercism solutions backup complete");
        // Ok(())
    }

    #[instrument(skip(this))]
    async fn backup_solutions(this: Arc<Self>, output_path: PathBuf) -> Result<()> {
        let mut task_pool = TaskPool::new();

        let mut page = 0;
        loop {
            let solutions = this.get_solutions_for_page(page).await?;

            if page == 0 {
                info!("Number of solutions to backup: {}", solutions.len());
            }

            if this.args.dry_run && event_enabled!(Level::INFO) {
                let solutions_list = solutions
                    .iter()
                    .map(|solution| format!("{}/{}", solution.track.name, solution.exercise.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                info!("Solutions to backup in page {page}: {solutions_list}");
            }

            if solutions.is_empty() || (this.args.dry_run && !event_enabled!(Level::INFO)) {
                break;
            }

            // Create track directories right away so that concurrent tasks don't end up trying
            // to create a directory multiple times.
            this.create_track_directories(&output_path, &solutions)
                .await?;

            if !this.args.dry_run || event_enabled!(Level::DEBUG) {
                for solution in solutions {
                    task_pool.spawn(Self::backup_solution(
                        this.clone(),
                        output_path.clone(),
                        solution,
                    ));
                }
            }

            page += 1;
        }

        task_pool
            .join(|| "errors detected while backing up solutions")
            .await
    }

    #[instrument(level = "debug", skip_all, fields(%solution.track.name, %solution.exercise.name))]
    async fn backup_solution(
        this: Arc<Self>,
        output_path: PathBuf,
        solution: Solution,
    ) -> Result<()> {
        todo!("back solution {solution:?} using {this:?} and {output_path:?}")

        // if !this.args.dry_run {
        //     debug!("Starting solution backup");
        // }
        // trace!(?solution);
        //
        // output_path.push(&solution.track.name);
        // output_path.push(&solution.exercise.name);
        // trace!(output_path = %output_path.display());
        //
        // if tokio::fs::metadata(&output_path.as_ref())
        //     .await
        //     .map(|meta| meta.is_dir())
        //     .unwrap_or(false)
        // {
        //     if args.force {
        //         trace!("Solution already exists on disk; cleaning up...");
        //         if !args.dry_run {
        //             tokio::fs::remove_dir_all(&output_path.as_ref())
        //                 .await
        //                 .with_context(|| {
        //                     format!("failed to clean up existing directory {}", output_path.display())
        //                 })?;
        //         }
        //     } else {
        //         trace!("Solution already exists on disk; skipping");
        //         return Ok(false);
        //     }
        // }
        //
        // if !args.dry_run {
        //     tokio::fs::create_dir_all(&output_path.as_ref())
        //         .await
        //         .with_context(|| {
        //             format!(
        //                 "failed to create destination directory for solution to {}/{}: {}",
        //                 solution.track.name,
        //                 solution.exercise.name,
        //                 output_path.display(),
        //             )
        //         })?;
        // }
        //
        // let files = {
        //     let _permit = limiter
        //         .acquire()
        //         .await
        //         .expect("failed to acquire limiter semaphore");
        //
        //     get_files_to_backup(&client, &solution).await?
        // };
        // if args.dry_run {
        //     debug!("Files to backup: {}", files.join(", "));
        // }
        //
        // if !args.dry_run || enabled!(Level::TRACE) {
        //     let mut downloads = JoinSet::new();
        //     for file in files {
        //         let client = client.clone();
        //         let solution = solution.clone();
        //         let output_path = output_path.clone();
        //         let limiter = limiter.clone();
        //         let dry_run = args.dry_run;
        //         downloads.spawn(async move {
        //             let _permit = limiter
        //                 .acquire()
        //                 .await
        //                 .expect("failed to acquire limiter semaphore");
        //
        //             download_one_file(client, &solution, file, &output_path, dry_run).await
        //         });
        //     }
        //
        //     wait_for_all(&mut downloads).await?;
        // }
        //
        // Ok(true)
    }

    #[instrument(skip(self))]
    async fn create_output_directory(&self, output_path: &Path) -> Result<()> {
        if !self.args.dry_run {
            fs::create_dir_all(output_path).await.with_context(|| {
                format!("failed to create output directory {}", output_path.display())
            })?;
        }

        Ok(())
    }

    #[instrument(skip(self), ret(level = "trace"))]
    async fn get_solutions_for_page(&self, page: i64) -> Result<Vec<Solution>> {
        let filters = solutions::Filters::builder()
            .status(self.args.status.into())
            .build();
        let paging = solutions::Paging::for_page(page);

        let _permit = self.limiter.get_permit();
        Ok(self
            .v2_client
            .get_solutions(Some(filters), Some(paging), None)
            .await
            .with_context(|| format!("failed to fetch solutions for page {page}"))?
            .results)
    }

    #[instrument(skip_all)]
    async fn create_track_directories(
        &self,
        output_path: &Path,
        solutions: &[Solution],
    ) -> Result<()> {
        if !self.args.dry_run {
            let track_names = solutions
                .iter()
                .map(|solution| solution.track.name.as_str())
                .collect::<HashSet<_>>();

            for track_name in track_names {
                let mut destination_path = output_path.to_path_buf();
                destination_path.push(track_name);

                fs::create_dir_all(&destination_path)
                    .await
                    .with_context(|| {
                        format!("failed to create directory for track {track_name}")
                    })?;
            }
        }

        Ok(())
    }

    // #[instrument(skip_all)]
    // async fn create_solution_directory(&self, solution_output_path: &Path) -> Option<Result<()>> {
    //     if fs::metadata(solution_output_path)
    //         .await
    //         .map(|meta| meta.is_dir())
    //         .unwrap_or(false)
    //     {
    //         if self.args.force {
    //             trace!("Solution already exists on disk; cleaning up...");
    //             if !self.args.dry_run {
    //                 fs::remove_dir_all(solution_output_path)
    //                     .await
    //                     .with_context(|| {
    //                         format!("failed to clean up existing directory {}", output_path.display())
    //                     })?;
    //             }
    //         } else {
    //             trace!("Solution already exists on disk; skipping");
    //             return Ok(false);
    //         }
    //     }
    //
    //     if !args.dry_run {
    //         tokio::fs::create_dir_all(&output_path.as_ref())
    //             .await
    //             .with_context(|| {
    //                 format!(
    //                     "failed to create destination directory for solution to {}/{}: {}",
    //                     solution.track.name,
    //                     solution.exercise.name,
    //                     output_path.display(),
    //                 )
    //             })?;
    //     }
    // }
}
