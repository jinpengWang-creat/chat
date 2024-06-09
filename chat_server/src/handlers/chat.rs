use axum::{response::IntoResponse, Extension, Json};

use crate::User;

pub async fn create_chat_handler() -> impl IntoResponse {
    // handle create chat here
    "create chat"
}

pub async fn list_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
    // handle list chat here
    Json(user)
}

pub async fn update_chat_handler() -> impl IntoResponse {
    // handle update chat here
    "update chat"
}

pub async fn delete_chat_handler() -> impl IntoResponse {
    // handle delete chat here
    "delete chat"
}
