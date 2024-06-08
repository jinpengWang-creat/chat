use axum::response::IntoResponse;

pub async fn signin_handler() -> impl IntoResponse {
    // handle signin here
    "signin"
}

pub async fn signup_handler() -> impl IntoResponse {
    // handle signup here
    "signup"
}
