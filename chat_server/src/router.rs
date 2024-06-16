use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};

use crate::{
    config::AppConfig,
    error::AppError,
    handlers::*,
    middlewares::{set_layer, verify_token},
    state::AppState,
};

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/chats", post(create_chat_handler).get(list_chat_handler))
        .route(
            "/chats/:id",
            get(get_chat_handler)
                .post(send_message_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/chats/:id/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state.clone());
    Ok(set_layer(app))
}
