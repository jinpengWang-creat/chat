use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{
    error::AppError,
    models::{Chat, CreateChat},
    state::AppState,
    User,
};

use super::AppJson;

pub async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    AppJson(create_chat): AppJson<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    // handle create chat here
    let chat = Chat::create_chat(create_chat, user.ws_id, &app_state.pool).await?;
    Ok(Json(chat))
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
