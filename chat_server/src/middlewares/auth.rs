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
