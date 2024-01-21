macro_rules! build_client {
    ($client_ty:ty, $http_client:ident, $credentials:ident, $api_base_url:ident) => {{
        let mut builder = <$client_ty>::builder();
        builder
            .http_client($http_client.clone())
            .credentials($credentials.clone());
        if let Some(api_base_url) = $api_base_url {
            builder.api_base_url(api_base_url);
        }
        builder.build()
    }};
}

// Downloads all files submitted for the given [`Solution`] in the given `output_path`.
//
// Whether an existing solution will be overwritten depends on the [`force`](BackupArgs::force) flag.
//
// # Return values
//
// | Outcome                                        | Return value |
// |------------------------------------------------|--------------|
// | Solution is downloaded successfully            | `Ok(true)`   |
// | Solution already exists on disk and is skipped | `Ok(false)`  |
// | Error occurs during a download                 | `Err(_)`     |
// #[instrument(
//     level = "debug",
//     skip_all,
//     fields(%solution.track.name, %solution.exercise.name)
// )]
// pub async fn download_one_solution(
//     client: api::v1::Client,
//     solution: Cow<'static, Solution>,
//     output_path: Cow<'static, PathBuf>,
//     args: &BackupArgs,
//     limiter: Arc<Semaphore>,
// ) -> Result<bool> {
//     if !args.dry_run {
//         debug!("Starting solution backup");
//     }
//     trace!(?solution);
//
//     let mut output_path = output_path;
//     output_path.to_mut().push(&solution.track.name);
//     output_path.to_mut().push(&solution.exercise.name);
//     trace!(output_path = %output_path.display());
//
//     if tokio::fs::metadata(&output_path.as_ref())
//         .await
//         .map(|meta| meta.is_dir())
//         .unwrap_or(false)
//     {
//         if args.force {
//             trace!("Solution already exists on disk; cleaning up...");
//             if !args.dry_run {
//                 tokio::fs::remove_dir_all(&output_path.as_ref())
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
//
//     let files = {
//         let _permit = limiter
//             .acquire()
//             .await
//             .expect("failed to acquire limiter semaphore");
//
//         get_files_to_backup(&client, &solution).await?
//     };
//     if args.dry_run {
//         debug!("Files to backup: {}", files.join(", "));
//     }
//
//     if !args.dry_run || enabled!(Level::TRACE) {
//         let mut downloads = JoinSet::new();
//         for file in files {
//             let client = client.clone();
//             let solution = solution.clone();
//             let output_path = output_path.clone();
//             let limiter = limiter.clone();
//             let dry_run = args.dry_run;
//             downloads.spawn(async move {
//                 let _permit = limiter
//                     .acquire()
//                     .await
//                     .expect("failed to acquire limiter semaphore");
//
//                 download_one_file(client, &solution, file, &output_path, dry_run).await
//             });
//         }
//
//         wait_for_all(&mut downloads).await?;
//     }
//
//     Ok(true)
// }

// Downloads the given `file` for the given [`Solution`] to the given `destination_path`.
// #[instrument(
//     level = "trace",
//     skip_all,
//     fields(%solution.track.name, %solution.exercise.name, file)
// )]
// pub async fn download_one_file(
//     client: api::v1::Client,
//     solution: &Solution,
//     file: String,
//     destination_path: &Path,
//     dry_run: bool,
// ) -> Result<()> {
//     let mut file_stream = client.get_file(&solution.uuid, &file).await;
//
//     let mut destination_file_path = destination_path.to_path_buf();
//     destination_file_path.extend(file.split('/'));
//     trace!(destination_file_path = %destination_file_path.display());
//
//     if !dry_run {
//         if let Some(parent) = destination_file_path.parent() {
//             tokio::fs::create_dir_all(parent).await.with_context(|| {
//                 format!(
//                     "failed to make sure parent of file {} exists",
//                     destination_file_path.display()
//                 )
//             })?;
//         }
//
//         let destination_file = tokio::fs::File::create(&destination_file_path)
//             .await
//             .with_context(|| {
//                 format!("failed to create local file {}", destination_file_path.display())
//             })?;
//         let mut destination_file = BufWriter::new(destination_file);
//
//         while let Some(bytes) = file_stream.next().await {
//             let bytes = bytes.with_context(|| {
//                 format!(
//                     "failed to download file {} in solution to exercise {} of track {}",
//                     file, solution.exercise.name, solution.track.name
//                 )
//             })?;
//             destination_file.write_all(&bytes).await.with_context(|| {
//                 format!("failed to write data to file {}", destination_file_path.display())
//             })?;
//         }
//
//         destination_file.flush().await.with_context(|| {
//             format!("failed to flush data to file {}", destination_file_path.display())
//         })?;
//     }
//
//     Ok(())
// }
