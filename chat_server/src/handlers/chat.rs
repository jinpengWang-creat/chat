use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    error::AppError,
    models::{Chat, CreateChat},
    state::AppState,
    User,
};

use super::AppJson;

pub async fn get_chat_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id as i64, &app_state.pool).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("Chat with id {} not found", id))),
    }
}

pub async fn create_chat_handler(
    State(app_state): State<AppState>,
    AppJson(create_chat): AppJson<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    // handle create chat here
    let chat = Chat::create_chat(create_chat, &app_state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = Chat::fetch_all_by_ws_id(user.ws_id as u64, &app_state.pool).await?;
    // handle list chat here
    Ok((StatusCode::OK, Json(chats)))
}

pub async fn update_chat_handler() -> impl IntoResponse {
    // handle update chat here
    "update chat"
}

pub async fn delete_chat_handler() -> impl IntoResponse {
    // handle delete chat here
    "delete chat"
}
