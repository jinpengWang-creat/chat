use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::request,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{error::AppError, state::AppState};

pub async fn verify_token(
    State(app_state): State<AppState>,
    AuthHeader(token): AuthHeader,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let user = app_state.dk.verify(&token)?;
    request.extensions_mut().insert(user);
    let ret = next.run(request).await;
    Ok(ret)
}

pub struct AuthHeader(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthHeader
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await {
            Ok(TypedHeader(Authorization(bearer))) => Ok(Self(bearer.token().to_string())),
            _ => Err(AppError::Unauthorized),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{config::AppConfig, User};

    use super::*;
    use anyhow::Result;
    use axum::{
        body::Body, http::StatusCode, middleware::from_fn_with_state, routing::get, Router,
    };
    use http_body_util::BodyExt;
    use request::Request;
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }
    #[tokio::test]
    async fn verify_token_middleware_should_work() -> Result<()> {
        let config = AppConfig::load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;

        let user = User::new(11, "j1im", "jim@122.com");
        let token = state.ek.sign(user)?;
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        let req = Request::builder()
            .method("GET")
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        let val = res.collect().await?.to_bytes().to_vec();
        assert_eq!(val, b"OK");

        // test without token
        let req = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // test with invalid token
        let req = Request::builder()
            .method("GET")
            .uri("/")
            .header("Authorization", "invalid_token")
            .body(Body::empty())?;

        let res = app.oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
