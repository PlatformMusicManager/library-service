use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use domain::models::db::user::{TrackPlatform, UserPlaylistWithTracks, UserWithPlaylists};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreatePlaylist {
    pub title: String,
}

pub async fn create_playlist(
    State(state): State<AppState>,
    Extension(user): Extension<UserWithPlaylists>,
    Json(body): Json<CreatePlaylist>,
) -> Result<Json<i64>, StatusCode> {
    let id = state
        .database
        .create_playlist(&body.title, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Invalidate user cache because playlists changed
    let _ = state.redis.remove_user(user.id).await;

    Ok(Json(id))
}

pub async fn get_playlist(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UserPlaylistWithTracks>, StatusCode> {
    let playlist = state
        .database
        .get_user_playlist_with_tracks(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(
        playlist.ok_or(StatusCode::NOT_FOUND)?
    ))
}

pub async fn delete_playlist(
    State(state): State<AppState>,
    Extension(user): Extension<UserWithPlaylists>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    // Check ownership
    if !user.playlists.iter().any(|p| p.id == id) {
        return Err(StatusCode::FORBIDDEN);
    }

    state
        .database
        .delete_playlist(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Invalidate user cache because playlists changed
    let _ = state.redis.remove_user(user.id).await;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct AddTrack {
    pub playlist_id: i64,
    pub track_id: i64,
    pub platform: TrackPlatform,
}

pub async fn add_track(
    State(state): State<AppState>,
    Extension(user): Extension<UserWithPlaylists>,
    Json(body): Json<AddTrack>,
) -> Result<Json<i64>, StatusCode> {
    // Check ownership of playlist
    if !user.playlists.iter().any(|p| p.id == body.playlist_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let id = state
        .database
        .add_track_to_playlist(body.playlist_id, body.track_id, body.platform)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(id))
}

pub async fn remove_track(
    State(state): State<AppState>,
    Extension(user): Extension<UserWithPlaylists>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    state
        .database
        .remove_track_from_playlist(id, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct MoveTrack {
    pub track_in_playlist_id: i64,
    pub new_position: i32,
}

pub async fn move_track(
    State(state): State<AppState>,
    Json(body): Json<MoveTrack>,
) -> Result<StatusCode, StatusCode> {
    state
        .database
        .change_track_position(body.track_in_playlist_id, body.new_position)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
