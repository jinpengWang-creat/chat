use axum::{extract::rejection::JsonRejection, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHash(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::JWTError),

    #[error("anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("json rejection error: {0}")]
    JsonRejection(#[from] JsonRejection),

    #[error("login failed: {0}")]
    LoginFailed(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::response::Json;

        let status = match &self {
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHash(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Jwt(_) => StatusCode::FORBIDDEN,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonRejection(_) => StatusCode::BAD_REQUEST,
            AppError::LoginFailed(_) => StatusCode::FORBIDDEN,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
