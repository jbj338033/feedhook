use crate::models::Channel;
use crate::notifier::{self, NewVideo};
use feed_rs::parser;
use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{error, info};

pub async fn run(
    pool: SqlitePool,
    client: Client,
    mut interval_rx: watch::Receiver<u64>,
) {
    let pool = Arc::new(pool);
    let client = Arc::new(client);
    let mut interval_secs = *interval_rx.borrow();

    loop {
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;

        if interval_rx.has_changed().unwrap_or(false) {
            interval_secs = *interval_rx.borrow_and_update();
            info!("polling interval changed to {interval_secs}s");
        }

        poll_all(&pool, &client).await;
    }
}

pub async fn poll_all(pool: &SqlitePool, client: &Client) {
    let channels: Vec<Channel> = match sqlx::query_as("SELECT * FROM channels")
        .fetch_all(pool)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            error!("failed to fetch channels: {e}");
            return;
        }
    };

    for channel in &channels {
        if let Err(e) = poll_channel(pool, client, channel).await {
            error!("failed to poll channel {}: {e}", channel.channel_id);
        }
    }
}

async fn poll_channel(
    pool: &SqlitePool,
    client: &Client,
    channel: &Channel,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        channel.channel_id
    );

    let body = client.get(&url).send().await?.bytes().await?;
    let feed = parser::parse(&body[..])?;

    for entry in &feed.entries {
        let video_id = entry
            .id
            .strip_prefix("yt:video:")
            .unwrap_or(&entry.id);

        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM videos WHERE video_id = ?)")
            .bind(video_id)
            .fetch_one(pool)
            .await?;

        if exists {
            continue;
        }

        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_default();

        let published_at = entry
            .published
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339();

        sqlx::query("INSERT INTO videos (channel_id, video_id, title, published_at) VALUES (?, ?, ?, ?)")
            .bind(&channel.channel_id)
            .bind(video_id)
            .bind(&title)
            .bind(&published_at)
            .execute(pool)
            .await?;

        info!("new video: {} - {}", channel.channel_name, title);

        notifier::send_discord(
            client,
            pool,
            &channel.webhook_url,
            &NewVideo {
                video_id: video_id.to_string(),
                title,
                channel_id: channel.channel_id.clone(),
                channel_name: channel.channel_name.clone(),
                published_at,
            },
        )
        .await;
    }

    Ok(())
}

pub async fn seed_existing_videos(pool: &SqlitePool, client: &Client, channel_id: &str) {
    let url = format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"
    );

    let Ok(resp) = client.get(&url).send().await else { return };
    let Ok(body) = resp.bytes().await else { return };
    let Ok(feed) = parser::parse(&body[..]) else { return };

    for entry in &feed.entries {
        let video_id = entry.id.strip_prefix("yt:video:").unwrap_or(&entry.id);
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_default();
        let published_at = entry
            .published
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339();

        let _ = sqlx::query("INSERT OR IGNORE INTO videos (channel_id, video_id, title, published_at) VALUES (?, ?, ?, ?)")
            .bind(channel_id)
            .bind(video_id)
            .bind(&title)
            .bind(&published_at)
            .execute(pool)
            .await;
    }
}
