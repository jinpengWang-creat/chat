use axum::response::IntoResponse;
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
    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::response::Json;

        let status = match &self {
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
