use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

use super::{ChatFile, Message};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListMessage {
    pub last_id: Option<u64>,
    pub limit: u64,
}

impl AppState {
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

    pub async fn list_message(
        &self,
        chat_id: u64,
        input: ListMessage,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as u64);
        let limit = input.limit;
        let messages: Vec<Message> = sqlx::query_as(
            "SELECT * FROM messages WHERE chat_id = $1 AND id < $2 ORDER BY id DESC LIMIT $3",
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::state::AppState;

    #[tokio::test]
    async fn create_message_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat_id = 1;
        let user_id = 1;

        // test content is empty
        let create_message = CreateMessage {
            content: "".to_string(),
            files: vec![],
        };

        let result = state.create_message(create_message, chat_id, user_id).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "create message error: Message content is empty"
        );

        // test file path is empty
        let create_message = CreateMessage {
            content: "Hello, World!".to_string(),
            files: vec!["".to_string()],
        };

        let result = state.create_message(create_message, chat_id, user_id).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "create message error: File path is empty"
        );

        // test invalid file path
        let invalid_path = "chat_file:1";
        let create_message = CreateMessage {
            content: "Hello, World!".to_string(),
            files: vec![invalid_path.to_string()],
        };

        let result = state.create_message(create_message, chat_id, user_id).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("chat file error: Invalid file path: {}", invalid_path)
        );

        // test file not found
        let create_message = CreateMessage {
            content: "Hello, World!".to_string(),
            files: vec!["/files/1/3es/32e/jis2234jisowe.txt".to_string()],
        };

        let result = state.create_message(create_message, chat_id, user_id).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "create message error: File not found"
        );

        // test create message
        let user_id = 1;
        let create_message = CreateMessage {
            content: "Hello, World!".to_string(),
            files: vec![],
        };

        let message = state
            .create_message(create_message, chat_id, user_id)
            .await
            .unwrap();
        assert_eq!(message.content, "Hello, World!");
        assert_eq!(message.files, Vec::<String>::new());
        Ok(())
    }

    #[tokio::test]
    async fn list_message_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat_id = 1;

        // test list message
        let input = ListMessage {
            last_id: None,
            limit: 6,
        };

        let messages = state.list_message(chat_id, input).await.unwrap();
        assert_eq!(messages.len(), 6);

        let input = ListMessage {
            last_id: Some(5),
            limit: 6,
        };

        let messages = state.list_message(chat_id, input).await.unwrap();
        assert_eq!(messages.len(), 4);

        Ok(())
    }
}
