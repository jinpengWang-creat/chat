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
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("create chat error: {0}")]
    CreateChat(String),

    #[error("chat not found error: {0}")]
    NotFound(String),

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

    #[error("Unauthorized")]
    Unauthorized,

    #[error("request header to str error: {0}")]
    RequestHeaderToStr(#[from] axum::http::header::ToStrError),

    #[error("multiple errors: {0}")]
    Multiple(#[from] axum::extract::multipart::MultipartError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::response::Json;

        let status = match &self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHash(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Jwt(_) => StatusCode::FORBIDDEN,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonRejection(_) => StatusCode::BAD_REQUEST,
            AppError::LoginFailed(_) => StatusCode::FORBIDDEN,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::RequestHeaderToStr(_) => StatusCode::BAD_REQUEST,
            AppError::CreateChat(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Multiple(_) => StatusCode::BAD_REQUEST,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidHeaderValue(_) => StatusCode::BAD_REQUEST,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
