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
    pub async fn create_chat(
        create_chat: CreateChat,
        ws_id: i64,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let chat: Chat = sqlx::query_as(
            "INSERT INTO chats (name, type, members, ws_id) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(&create_chat.name)
        .bind(&create_chat.chat_type)
        .bind(&create_chat.members)
        .bind(ws_id)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }
}
