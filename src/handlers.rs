use crate::error::AppError;
use crate::models::{Channel, CreateChannel, NotificationLog, Settings};
use crate::poller;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

pub async fn list_channels(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Channel>>, AppError> {
    let channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels ORDER BY id DESC")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(channels))
}

pub async fn create_channel(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateChannel>,
) -> Result<(StatusCode, Json<Channel>), AppError> {
    let channel = sqlx::query_as::<_, Channel>(
        "INSERT INTO channels (channel_id, channel_name, webhook_url) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&body.channel_id)
    .bind(&body.channel_name)
    .bind(&body.webhook_url)
    .fetch_one(&state.pool)
    .await?;

    poller::seed_existing_videos(&state.pool, &state.client, &body.channel_id).await;

    Ok((StatusCode::CREATED, Json(channel)))
}

pub async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM channels WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Settings>, AppError> {
    let row: (String,) =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'polling_interval'")
            .fetch_one(&state.pool)
            .await?;
    let polling_interval = row.0.parse::<u64>().unwrap_or(300);
    Ok(Json(Settings { polling_interval }))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Settings>,
) -> Result<Json<Settings>, AppError> {
    sqlx::query("UPDATE settings SET value = ? WHERE key = 'polling_interval'")
        .bind(body.polling_interval.to_string())
        .execute(&state.pool)
        .await?;
    let _ = state.interval_tx.send(body.polling_interval);
    Ok(Json(body))
}

pub async fn list_logs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NotificationLog>>, AppError> {
    let logs = sqlx::query_as::<_, NotificationLog>(
        "SELECT * FROM notification_log ORDER BY sent_at DESC LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(logs))
}

pub async fn trigger_poll(
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.clone();
    let client = state.client.clone();
    tokio::spawn(async move {
        poller::poll_all(&pool, &client).await;
    });
    Ok(StatusCode::ACCEPTED)
}
