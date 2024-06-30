use axum::{
    extract::State,
    response::sse::{Event, Sse},
    Extension,
};
use chat_core::User;
use futures::stream::{self, Stream};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tracing::info;

use crate::{AppEvent, AppState};
const MAX_CHANNEL_SIZE: usize = 100;
pub async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_id = user.id as u64;
    let receive = if let Some(sender) = state.users.get(&user_id) {
        info!("user {} subscribed", user_id);
        sender.subscribe()
    } else {
        info!("user {} created", user_id);
        let (sender, receive) = broadcast::channel(MAX_CHANNEL_SIZE);
        state.users.insert(user_id, sender);
        receive
    };

    let stream = BroadcastStream::new(receive)
        .filter_map(|v| v.ok())
        .merge(stream::repeat_with(|| Arc::new(AppEvent::Alive)).throttle(Duration::from_secs(1)))
        .map(|e| {
            let name = match e.as_ref() {
                AppEvent::NewChat(_) => "NewChat",
                AppEvent::AddToChat(_) => "AddToChat",
                AppEvent::RemoveFromChat(_) => "RemoveFromChat",
                AppEvent::NewMessage(_) => "NewMessage",
                AppEvent::Alive => "Alive",
            };
            let data = serde_json::to_string(&e).expect("Failed to serialize event");
            Ok(Event::default().event(name).data(data))
        });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .event(Event::default().event("keep-alive").data("1"))
            .text("keep-alive-text"),
    )
}
