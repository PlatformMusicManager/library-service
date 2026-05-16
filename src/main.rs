mod routes;

use crate::routes::playlist::{add_track, create_playlist, delete_playlist, get_playlist, move_track, remove_track};
use crate::routes::user::get_me;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use cache_lib::RedisClient;
use chrono::Duration;
use database_lib::client::PostgresDb;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use utils_lib::auth_layer::{AuthLayer, AuthState};
use utils_lib::jwt::JwtClient;
use utils_lib::parse_env::parse_env;

#[derive(Clone)]
struct AppState {
    pub database: PostgresDb,
    pub redis: RedisClient,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let redis_url = parse_env("REDIS_URL");
    let user_cache_ttl: i64 = parse_env("USER_CACHE_TTL");
    let session_cache_ttl: u64 = parse_env("SESSION_CACHE_TTL");
    let verify_ttl_s: u64 = parse_env("VERIFY_TTL");
    let verify_ttl = Duration::seconds(verify_ttl_s as i64);
    let verify_attempts: u8 = parse_env("VERIFY_ATTEMPTS");

    let jwt_secret = parse_env("JWT_SECRET");

    let access_token_ttl_s: i64 = parse_env("ACCESS_TOKEN_TTL");
    let refresh_token_ttl_s: i64 = parse_env("REFRESH_TOKEN_TTL");

    let database_url: String = env::var("DATABASE_URL").unwrap();

    let database = PostgresDb::new(database_url, Duration::new(0, 0).unwrap()).await;

    let redis_client = RedisClient::new(
        redis_url,
        session_cache_ttl,
        user_cache_ttl,
        verify_ttl_s,
        verify_attempts,
    );

    let auth_state = AuthState {
        database: database.clone(),
        redis: redis_client.clone(),
        jwt: Arc::new(JwtClient::new(
            jwt_secret,
            verify_ttl,
            Duration::seconds(access_token_ttl_s),
            Duration::seconds(refresh_token_ttl_s),
        )),
    };

    let app_state = AppState {
        database,
        redis: redis_client,
    };

    let routes = Router::new()
        .route("/me", get(get_me))
        .route("/playlist", post(create_playlist))
        .route("/playlist/{id}", get(get_playlist))
        .route("/playlist/{id}", delete(delete_playlist))
        .route("/playlist/track", post(add_track))
        .route("/playlist/track/{id}", delete(remove_track))
        .route("/playlist/track/move", put(move_track))
        .with_state(app_state);

    let app = Router::new()
        .nest("/api/library", routes)
        .layer(AuthLayer { state: auth_state });

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
