use axum::{
    extract::{Path, Request, State},
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use chat_core::User;

use crate::{error::AppError, state::AppState};

pub async fn verify_is_chat_member(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<u64>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    if !app_state.is_chat_member(id, user.id as u64).await? {
        return Err(AppError::Unauthorized(
            "You are not a member of this chat".to_string(),
        ));
    }
    let ret = next.run(request).await;
    Ok(ret)
}

#[cfg(test)]
mod test {

    use super::*;
    use anyhow::Result;
    use axum::{
        body::Body, http::StatusCode, middleware::from_fn_with_state, routing::get, Router,
    };
    use chat_core::verify_token;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }
    #[tokio::test]
    async fn verify_chat_middleware_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let user = state.find_user_by_email("test1@none.org").await?.unwrap();
        let token = state.ek.sign(user)?;
        let app = Router::new()
            .route("/:id", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_is_chat_member))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .with_state(state.clone());

        let req = Request::builder()
            .method("GET")
            .uri("/2")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::OK);

        let val = res.collect().await?.to_bytes().to_vec();
        assert_eq!(val, b"OK");

        // test without token
        let user = state.find_user_by_email("test5@none.org").await?.unwrap();
        let token = state.ek.sign(user)?;
        let req = Request::builder()
            .method("GET")
            .uri("/2")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
