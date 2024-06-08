use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tracing::info;

use crate::{config::AppConfig, handlers::*};

#[derive(Debug, Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

#[derive(Debug)]
struct AppStateInner {
    config: AppConfig,
}

impl AppState {
    fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

pub async fn run() -> Result<()> {
    let config = AppConfig::load()?;

    let state = AppState::new(config);

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
    let ip = &state.config.server.ip;
    let port = &state.config.server.port;
    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
