use axum::{
    async_trait,
    extract::{FromRequestParts, Request, State},
    http::{request, StatusCode},
    middleware::Next,
    response::{IntoResponse as _, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use super::TokenVerifier;

pub async fn verify_token<T>(
    State(app_state): State<T>,
    AuthHeader(token): AuthHeader,
    mut request: Request,
    next: Next,
) -> Response
where
    T: TokenVerifier + Clone + Send + Sync + 'static,
{
    let user = app_state.verify(&token);
    let Ok(user) = user else {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    };
    request.extensions_mut().insert(user);
    next.run(request).await
}

pub struct AuthHeader(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthHeader
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await {
            Ok(TypedHeader(Authorization(bearer))) => Ok(Self(bearer.token().to_string())),
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        utils::{DecodingKey, EncodingKey},
        User,
    };

    use super::*;
    use anyhow::Result;
    use axum::{
        body::Body, http::StatusCode, middleware::from_fn_with_state, response::IntoResponse,
        routing::get, Router,
    };
    use http_body_util::BodyExt;
    use request::Request;
    use tower::ServiceExt;

    #[derive(Clone)]
    pub struct AppState(Arc<AppStateInner>);

    pub struct AppStateInner {
        pub pk: DecodingKey,
        pub ek: EncodingKey,
    }

    impl TokenVerifier for AppState {
        type Error = anyhow::Error;
        fn verify(&self, token: &str) -> Result<User, Self::Error> {
            self.0.pk.verify(token)
        }
    }

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }
    #[tokio::test]
    async fn verify_token_middleware_should_work() -> Result<()> {
        let ek = EncodingKey::load(include_str!("../../fixtures/encoding.pem"))?;
        let pk = DecodingKey::load(include_str!("../../fixtures/decoding.pem"))?;
        let state = AppState(Arc::new(AppStateInner { pk, ek }));
        let user = User::new(11, "j1im", "jim@122.com", 0);
        let token = state.0.ek.sign(user)?;
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
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
