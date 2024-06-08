mod auth;
mod chat;
mod message;

pub use auth::*;
use axum::response::IntoResponse;
pub use chat::*;
pub use message::*;

pub async fn index_handler() -> impl IntoResponse {
    "Hello, World!"
}
