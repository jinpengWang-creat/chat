use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{error::AppError, state::AppState, User};

pub async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_chat_users(user.ws_id).await?;
    Ok(Json(users))
}
