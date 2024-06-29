use std::sync::Arc;

use chat_core::{Chat, Message};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tokio_stream::StreamExt;
use tracing::warn;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message),
}
// PERFORM pg_notify('chat_updated', json_build_object('op', TG_OP,'old', OLD, 'new', NEW )::text);
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ChatUpdated {
    pub op: String,
    pub old: Option<Chat>,
    pub new: Option<Chat>,
}

// PERFORM pg_notify('chat_message_created', row_to_json(NEW)::text);
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ChatMessageCreated {
    pub chat: Chat,
    pub message: Message,
}

#[derive(Debug)]
pub struct Notification {
    pub user_ids: Vec<u64>,
    pub event: Arc<AppEvent>,
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
                    let notification =
                        Notification::load(notification.channel(), notification.payload())?;
                    let user_map = state.users.clone();
                    for user_id in notification.user_ids {
                        if let Some(sender) = user_map.get(&user_id) {
                            if let Err(err) = sender.send(notification.event.clone()) {
                                warn!("Failed to send notification to user {}: {:?}", user_id, err);
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("error: {:?}", err);
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

impl Notification {
    fn load(channel: &str, payload: &str) -> anyhow::Result<Self> {
        match channel {
            "chat_updated" => {
                let chat_updated: ChatUpdated = serde_json::from_str(payload)?;
                let user_ids = get_affected_chat_user_ids(
                    chat_updated.old.as_ref(),
                    chat_updated.new.as_ref(),
                );
                let event = match chat_updated.op.as_str() {
                    "INSERT" => AppEvent::NewChat(chat_updated.new.expect("new chat is None")),
                    "UPDATE" => AppEvent::AddToChat(chat_updated.new.expect("new chat is None")),
                    "DELETE" => {
                        AppEvent::RemoveFromChat(chat_updated.old.expect("old chat is None"))
                    }
                    _ => anyhow::bail!("unknown operation: {}", chat_updated.op),
                };
                let event = Arc::new(event);
                Ok(Self { user_ids, event })
            }
            "chat_message_created" => {
                let chat_message_created: ChatMessageCreated = serde_json::from_str(payload)?;
                let user_ids = chat_message_created
                    .chat
                    .members
                    .iter()
                    .map(|id| *id as u64)
                    .collect();
                let event = Arc::new(AppEvent::NewMessage(chat_message_created.message));
                Ok(Self { user_ids, event })
            }
            _ => anyhow::bail!("unknown channel: {}", channel),
        }
    }
}

fn get_affected_chat_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> Vec<u64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            let mut old_user_ids = old.members.iter().map(|id| *id as u64).collect::<Vec<_>>();
            let mut new_user_ids = new.members.iter().map(|id| *id as u64).collect::<Vec<_>>();
            // get the union of old_user_ids and new_user_ids and distinct them
            old_user_ids.append(&mut new_user_ids);
            old_user_ids.sort_unstable();
            old_user_ids.dedup();
            old_user_ids
        }
        (Some(old), None) => old.members.iter().map(|id| *id as u64).collect::<Vec<_>>(),
        (None, Some(new)) => new.members.iter().map(|id| *id as u64).collect::<Vec<_>>(),
        (None, None) => Vec::new(),
    }
}
