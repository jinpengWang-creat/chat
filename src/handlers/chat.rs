use axum::response::IntoResponse;

pub async fn create_chat_handler() -> impl IntoResponse {
    // handle create chat here
    "create chat"
}

pub async fn list_chat_handler() -> impl IntoResponse {
    // handle list chat here
    "list chat"
}

pub async fn update_chat_handler() -> impl IntoResponse {
    // handle update chat here
    "update chat"
}

pub async fn delete_chat_handler() -> impl IntoResponse {
    // handle delete chat here
    "delete chat"
}
