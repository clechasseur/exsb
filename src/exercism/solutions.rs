//! Helpers for getting solution information from the Exercism API.

use mini_exercism::api;

/// Returns the name of all files submitted in the given solution.
pub async fn get_solution_files(
    client: &api::v1::Client,
    solution_uuid: &str,
) -> crate::Result<Vec<String>> {
    Ok(client.get_solution(solution_uuid).await?.solution.files)
}
