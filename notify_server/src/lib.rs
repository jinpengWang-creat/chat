use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{Chat, Message};
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

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/events", get(sse_handler))
}

async fn index() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut lisitener =
        PgListener::connect("postgresql://fandream:fandream@localhost:5432/chat").await?;

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
