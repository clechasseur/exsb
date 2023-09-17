use mini_exercism::api;
use mini_exercism::api::v2::{ExerciseFilters, Solution, TrackFilters};
use mini_exercism::api::v2::TrackStatusFilter::Joined;

pub async fn get_joined_tracks(client: &api::v2::Client) -> crate::Result<Vec<String>> {
    let filters = TrackFilters::builder()
        .status(Joined)
        .build();
    let tracks = client.get_tracks(Some(filters)).await?.tracks;

    Ok(tracks.into_iter().map(|track| track.name).collect())
}

pub async fn get_solutions(client: &api::v2::Client, track: &str) -> crate::Result<Vec<Solution>> {
    let filters = ExerciseFilters::builder()
        .include_solutions(true)
        .build();

    Ok(client.get_exercises(track, Some(filters)).await?.solutions)
}
