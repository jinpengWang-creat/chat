use axum::response::IntoResponse;

pub async fn send_message_handler() -> impl IntoResponse {
    // handle send message here
    "send message"
}

pub async fn list_message_handler() -> impl IntoResponse {
    // handle list message here
    "list message"
}
