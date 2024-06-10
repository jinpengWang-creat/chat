use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;

use super::{Chat, ChatType};

#[derive(Debug, Deserialize)]
pub struct CreateChat {
    pub name: String,
    pub chat_type: ChatType,
    pub members: Vec<i64>,
}

impl Chat {
    pub async fn create_chat(create_chat: CreateChat, pool: &PgPool) -> Result<Self, AppError> {
        let chat: Chat = sqlx::query_as(
            "INSERT INTO chats (name, chat_type, members) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&create_chat.name)
        .bind(&create_chat.chat_type)
        .bind(&create_chat.members)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }
}
