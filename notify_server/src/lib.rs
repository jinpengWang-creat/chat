mod config;
mod error;
mod notify;
pub use config::AppConfig;
pub use notify::AppEvent;

use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{verify_token, DecodingKey, TokenVerifier, User};
use dashmap::DashMap;
use error::AppError;

use sse::sse_handler;
use tokio::sync::broadcast;

mod sse;

pub type UserMap = DashMap<u64, broadcast::Sender<Arc<AppEvent>>>;

const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub pk: DecodingKey,
    pub users: Arc<UserMap>,
    pub config: AppConfig,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config)?;
    notify::setup_pg_listener(state.clone()).await?;

    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index))
        .with_state(state.clone());

    Ok(router)
}

async fn index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl TokenVerifier for AppState {
    type Error = anyhow::Error;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        self.0.pk.verify(token)
    }
}

impl AppState {
    pub fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;

        Ok(Self(Arc::new(AppStateInner {
            pk,
            config,
            users: Arc::new(DashMap::default()),
        })))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
