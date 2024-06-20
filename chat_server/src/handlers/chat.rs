use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    error::AppError,
    models::{CreateChat, UpdateChat},
    state::AppState,
    User,
};

use super::AppJson;

pub async fn get_chat_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = app_state.get_chat_by_id(id as i64).await?;
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
    let chat = app_state.create_chat(create_chat).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = app_state.fetch_chats_by_ws_id(user.ws_id as u64).await?;
    // handle list chat here
    Ok((StatusCode::OK, Json(chats)))
}

pub async fn update_chat_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    AppJson(update_chat): AppJson<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = app_state.update_chat(id as i64, update_chat).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub async fn delete_chat_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    app_state.delete_chat(id as i64).await?;
    Ok(StatusCode::OK)
}
