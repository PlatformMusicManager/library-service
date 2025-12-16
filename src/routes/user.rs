use axum::{Extension, Json};
use domain::models::db::user::UserWithPlaylists;

pub async fn get_me(Extension(user): Extension<UserWithPlaylists>) -> Json<UserWithPlaylists> {
    Json(user)
}
