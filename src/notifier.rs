use reqwest::Client;
use serde_json::json;
use sqlx::SqlitePool;
use std::time::Duration;
use tracing::warn;

pub struct NewVideo {
    pub video_id: String,
    pub title: String,
    pub channel_id: String,
    pub channel_name: String,
    pub published_at: String,
}

pub async fn send_discord(client: &Client, pool: &SqlitePool, webhook_url: &str, video: &NewVideo) {
    let payload = json!({
        "content": format!("https://www.youtube.com/watch?v={}", video.video_id)
    });

    let (status, error_message) = match client.post(webhook_url).json(&payload).send().await {
        Ok(resp) if resp.status().is_success() => ("success".to_string(), None),
        Ok(resp) if resp.status() == 429 => {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(5.0);
            warn!("discord rate limited, retrying after {retry_after}s");
            tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;
            match client.post(webhook_url).json(&payload).send().await {
                Ok(r) if r.status().is_success() => ("success".to_string(), None),
                Ok(r) => ("failed".to_string(), Some(format!("status {}", r.status()))),
                Err(e) => ("failed".to_string(), Some(e.to_string())),
            }
        }
        Ok(resp) => (
            "failed".to_string(),
            Some(format!("status {}", resp.status())),
        ),
        Err(e) => ("failed".to_string(), Some(e.to_string())),
    };

    let _ = sqlx::query(
        "INSERT INTO notification_log (video_id, channel_id, webhook_url, status, error_message) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&video.video_id)
    .bind(&video.channel_id)
    .bind(webhook_url)
    .bind(&status)
    .bind(&error_message)
    .execute(pool)
    .await;
}
