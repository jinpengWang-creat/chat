use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{error::AppError, models::Workspace, state::AppState, User};

pub async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = Workspace::fetch_all_chat_users(user.ws_id, &state.pool).await?;
    Ok(Json(users))
}
