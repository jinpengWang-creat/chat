use axum::{
    routing::{get, post},
    Router,
};

use crate::{config::AppConfig, error::AppError, handlers::*, state::AppState};

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", post(create_chat_handler).get(list_chat_handler))
        .route(
            "/chat/:id",
            post(send_message_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route("/chat/:id/messages", get(list_message_handler))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api);
    Ok(app)
}
