use mini_exercism::api;
use mini_exercism::api::v2::TrackStatusFilter::Joined;
use mini_exercism::api::v2::{ExerciseFilters, Solution, TrackFilters};

pub async fn get_joined_tracks(client: &api::v2::Client) -> crate::Result<Vec<String>> {
    let filters = TrackFilters::builder().status(Joined).build();
    let tracks = client.get_tracks(Some(filters)).await?.tracks;

    Ok(tracks.into_iter().map(|track| track.name).collect())
}

pub async fn get_solutions<T>(client: &api::v2::Client, track: T) -> crate::Result<Vec<Solution>>
where
    T: AsRef<str>,
{
    let filters = ExerciseFilters::builder().include_solutions(true).build();

    Ok(client
        .get_exercises(track.as_ref(), Some(filters))
        .await?
        .solutions)
}
