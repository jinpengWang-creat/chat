mod auth;
mod chat;
mod message;
mod workspace;

pub use auth::*;
use axum::response::IntoResponse;
use axum_macros::FromRequest;
pub use chat::*;
pub use message::*;
pub use workspace::*;

use crate::error::AppError;

pub async fn index_handler() -> impl IntoResponse {
    "Hello, World!"
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);
