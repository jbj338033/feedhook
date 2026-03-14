use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub enum AppError {
    Db(sqlx::Error),
    Http(reqwest::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("database error: {e}")),
            Self::Http(e) => (StatusCode::BAD_GATEWAY, format!("http error: {e}")),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}
