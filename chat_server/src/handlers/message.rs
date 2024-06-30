use std::fs;

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;
use tracing::{info, warn};

use crate::{
    error::AppError,
    models::{ChatFile, CreateMessage, ListMessage},
    state::AppState,
};

use super::AppJson;

pub async fn send_message_handler(
    Extension(user): Extension<User>,
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    AppJson(create_message): AppJson<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    // handle send message here
    let message = app_state
        .create_message(create_message, id, user.id as u64)
        .await?;
    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn list_message_handler(
    Path(id): Path<u64>,
    State(app_state): State<AppState>,
    Query(list_message): Query<ListMessage>,
) -> Result<impl IntoResponse, AppError> {
    let messages = app_state.list_message(id, list_message).await?;
    Ok((StatusCode::OK, Json(messages)))
}

pub async fn file_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    Path((ws_id, path)): Path<(u64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id as i64 {
        return Err(AppError::Unauthorized);
    }

    let base_dir = app_state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound(format!("file not found - {:?}", path)));
    }
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(mime.to_string().as_str())?,
    );
    Ok((headers, body))
}

pub async fn upload_handler(
    Extension(user): Extension<User>,
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = &app_state.config.server.base_dir;
    let mut paths = vec![];
    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().map(String::from);
        let name = field.name().map(String::from);
        let bytes = field.bytes().await;
        if let (Some(filename), Ok(bytes)) = (filename.clone(), bytes) {
            let file = ChatFile::new(&filename, &bytes, ws_id as u64);
            let path = file.path(base_dir);
            if !path.exists() {
                fs::create_dir_all(path.parent().unwrap())?;
                fs::write(&path, &bytes)?;
            } else {
                info!("File already exists - {:?}", filename);
            }
            paths.push(file.url());
        } else {
            warn!(
                "Failed to read multipart field - name: {:?}, file name: {:?}",
                name, filename
            );
        }
    }
    // handle upload here
    Ok(Json(paths))
}
