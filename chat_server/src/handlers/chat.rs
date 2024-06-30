use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;

use crate::{
    error::AppError,
    models::{CreateChat, UpdateChat},
    state::AppState,
};

use super::AppJson;

#[utoipa::path(get, path = "/api/chats/{id}",
responses(
    (status = 200, description = "get chat in successful", body = Chat),
),
security(
    ("Authorization" = [])
)
)]
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

#[utoipa::path(post, path = "/api/chats",
request_body(content = CreateChat, description = "Create chat details"),
responses(
    (status = 200, description = "get chat list in successful", body = Chat),
),
security(
    ("Authorization" = [])
))]
pub async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    AppJson(mut create_chat): AppJson<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    // if user is not in create_chat.members, add self to create_chat.members
    if !create_chat.members.contains(&user.id) {
        create_chat.members.push(user.id);
    }
    // if the ws_id of user is not match with create_chat.ws_id, return error
    if user.ws_id != create_chat.ws_id {
        return Err(AppError::Unauthorized("ws_id does not match".to_string()));
    }
    // handle create chat here
    let chat = app_state.create_chat(create_chat).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

#[utoipa::path(get, path = "/api/chats",
responses(
    (status = 200, description = "get chat list in successful", body = Vec<Chat>),
),
security(
    ("Authorization" = [])
))]
pub async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = app_state.fetch_chats_by_ws_id(user.ws_id as u64).await?;
    // handle list chat here
    Ok((StatusCode::OK, Json(chats)))
}

#[utoipa::path(patch, path = "/api/chats/{id}",
request_body(content = UpdateChat, description = "Update chat details"),
responses(
    (status = 200, description = "update chat in successful", body = Chat),
),
security(
    ("Authorization" = [])
))]
pub async fn update_chat_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    AppJson(update_chat): AppJson<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = app_state.update_chat(id as i64, update_chat).await?;
    Ok((StatusCode::OK, Json(chat)))
}

#[utoipa::path(delete, path = "/api/chats/{id}",
responses(
    (status = 200, description = "delete chat in successful", body = Chat),
),
security(
    ("Authorization" = [])
))]
pub async fn delete_chat_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    app_state.delete_chat(id as i64).await?;
    Ok(StatusCode::OK)
}
