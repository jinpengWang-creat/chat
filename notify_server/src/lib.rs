mod config;
mod error;
mod notify;
use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{verify_token, Chat, DecodingKey, Message, TokenVerifier, User};
use config::AppConfig;
use error::AppError;
use futures::StreamExt;
use sqlx::postgres::PgListener;
use sse::sse_handler;

mod sse;

pub enum Event {
    NewChat(Chat),
    AddtoChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}

const INDEX_HTML: &str = include_str!("../index.html");

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub pk: DecodingKey,
    pub config: AppConfig,
}

pub fn get_router_with_state() -> Result<(Router, AppState), AppError> {
    let config = AppConfig::load()?;
    let state = AppState::try_new(config)?;
    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index));
    Ok((router, state))
}

async fn index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let mut lisitener = PgListener::connect(&state.config.server.db_url).await?;

    lisitener.listen("chat_updated").await?;
    lisitener.listen("chat_message_created").await?;

    let mut pg_stream = lisitener.into_stream();

    tokio::spawn(async move {
        while let Some(notification) = pg_stream.next().await {
            match notification {
                Ok(notification) => {
                    println!("notification: {:?}", notification);
                }
                Err(err) => {
                    eprintln!("error: {:?}", err);
                }
            }
        }
    });
    Ok(())
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

        Ok(Self(Arc::new(AppStateInner { pk, config })))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
