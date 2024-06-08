use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    error::AppError,
    models::{UserLogin, UserRegister},
    state::AppState,
    User,
};

use super::AppJson;

pub async fn signin_handler(
    State(state): State<AppState>,
    AppJson(input): AppJson<UserLogin>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, token))
        }
        None => Err(AppError::LoginFailed(
            "Invalid email or password".to_string(),
        )),
    }
}

pub async fn signup_handler(
    State(state): State<AppState>,
    AppJson(input): AppJson<UserRegister>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create_user(&input, &state.pool).await?;
    let token = state.ek.sign(user)?;
    Ok((StatusCode::CREATED, token))
}
