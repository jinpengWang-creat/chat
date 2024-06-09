use axum::{extract::rejection::JsonRejection, response::IntoResponse};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

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

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),
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
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
