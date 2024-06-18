use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Extension,
};

use crate::{error::AppError, state::AppState, User};

pub async fn send_message_handler() -> impl IntoResponse {
    // handle send message here
    "send message"
}

pub async fn list_message_handler() -> impl IntoResponse {
    // handle list message here
    "list message"
}

#[allow(dead_code)]
pub async fn upload_handler(
    Extension(_user): Extension<User>,
    State(_app_state): State<AppState>,
    mut _multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // handle upload here
    Ok("upload")
}
