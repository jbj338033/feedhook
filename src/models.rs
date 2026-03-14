use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize)]
pub struct Channel {
    pub id: i64,
    pub channel_id: String,
    pub channel_name: String,
    pub webhook_url: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateChannel {
    pub channel_id: String,
    pub channel_name: String,
    pub webhook_url: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct NotificationLog {
    pub id: i64,
    pub video_id: String,
    pub channel_id: String,
    pub webhook_url: String,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub polling_interval: u64,
}
