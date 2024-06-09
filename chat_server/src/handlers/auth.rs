use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{SigninUser, SignupUser},
    state::AppState,
    User,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

use super::AppJson;

pub async fn signin_handler(
    State(state): State<AppState>,
    AppJson(input): AppJson<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })))
        }
        None => Err(AppError::LoginFailed(
            "Invalid email or password".to_string(),
        )),
    }
}

pub async fn signup_handler(
    State(state): State<AppState>,
    AppJson(input): AppJson<SignupUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create_user(&input, &state.pool).await?;
    let token = state.ek.sign(user)?;
    Ok((StatusCode::CREATED, Json(AuthOutput { token })))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::AppConfig;
    use anyhow::Result;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let name = "tom";
        let email = "tom@123.com";
        let password = "1qa2ws3ed";
        let input = SignupUser::new(name, email, password);

        let res = signup_handler(State(state.clone()), AppJson(input))
            .await?
            .into_response();

        assert_eq!(res.status(), StatusCode::CREATED);
        let token = res.into_body();
        let bytes = token.collect().await?.to_bytes();
        let token = String::from_utf8(bytes.to_vec())?;
        assert_ne!(token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let name = "tom";
        let email = "tom@123.com";
        let password = "1qa2ws3ed";
        let input = SignupUser::new(name, email, password);
        signup_handler(State(state.clone()), AppJson(input)).await?;

        let input = SigninUser::new(email, password);
        let res = signin_handler(State(state.clone()), AppJson(input))
            .await?
            .into_response();

        assert_eq!(res.status(), StatusCode::OK);
        let token = res.into_body();
        let bytes = token.collect().await?.to_bytes();
        let token = String::from_utf8(bytes.to_vec())?;
        assert_ne!(token, "");
        Ok(())
    }
}
