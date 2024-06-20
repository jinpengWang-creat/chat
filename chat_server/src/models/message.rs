use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{error::AppError, state::AppState};

use super::{ChatFile, Message};

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn create_message(
        &self,
        create_message: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        if create_message.content.is_empty() {
            return Err(AppError::CreateMessage(
                "Message content is empty".to_string(),
            ));
        }

        let base_dir = &self.config.server.base_dir;
        for file in &create_message.files {
            if file.is_empty() {
                return Err(AppError::CreateMessage("File path is empty".to_string()));
            }
            let chat_file = ChatFile::from_str(file)?;
            if !chat_file.path(base_dir).exists() {
                return Err(AppError::CreateMessage("File not found".to_string()));
            }
        }

        let message: Message = sqlx::query_as(
            "INSERT INTO messages (chat_id, sender_id, content, files) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(create_message.content)
        .bind(create_message.files)
        .fetch_one(&self.pool)
        .await?;
        Ok(message)
    }
}
