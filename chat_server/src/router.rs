use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use chat_core::verify_token;

use crate::{
    config::AppConfig,
    error::AppError,
    handlers::*,
    middlewares::{set_layer, verify_is_chat_member},
    state::AppState,
};

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let chats = Router::new()
        .route(
            "/:id",
            get(get_chat_handler)
                .post(send_message_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/:id/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_is_chat_member))
        .route("/", post(create_chat_handler).get(list_chat_handler));
    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/upload", post(upload_handler))
        .route("/files/:ws_id/*path", get(file_handler))
        .nest("/chats", chats)
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state.clone());
    Ok(set_layer(app))
}
