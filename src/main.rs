mod config;
mod db;
mod error;
mod handlers;
mod models;
mod notifier;
mod poller;

use axum::Router;
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use reqwest::Client;
use rust_embed::Embed;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;

#[derive(Embed)]
#[folder = "web/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub client: Client,
    pub interval_tx: watch::Sender<u64>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = config::Config::from_env();
    let pool = db::init(&cfg.database_url)
        .await
        .expect("failed to init db");

    let interval: u64 =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'polling_interval'")
            .fetch_one(&pool)
            .await
            .ok()
            .and_then(|v: String| v.parse().ok())
            .unwrap_or(300);

    let (interval_tx, interval_rx) = watch::channel(interval);
    let client = Client::new();

    let state = Arc::new(AppState {
        pool: pool.clone(),
        client: client.clone(),
        interval_tx,
    });

    tokio::spawn(poller::run(pool, client, interval_rx));

    let app = Router::new()
        .route("/api/channels", get(handlers::list_channels))
        .route("/api/channels", post(handlers::create_channel))
        .route("/api/channels/{id}", delete(handlers::delete_channel))
        .route("/api/settings", get(handlers::get_settings))
        .route("/api/settings", put(handlers::update_settings))
        .route("/api/logs", get(handlers::list_logs))
        .route("/api/poll", post(handlers::trigger_poll))
        .fallback(static_handler)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.port);
    info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.to_string())], file.data).into_response()
        }
        None => match Assets::get("index.html") {
            Some(file) => {
                let html = String::from_utf8_lossy(&file.data).into_owned();
                Html(html).into_response()
            }
            None => (axum::http::StatusCode::NOT_FOUND, "not found").into_response(),
        },
    }
}
