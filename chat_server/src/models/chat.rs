use chat_core::Chat;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize, Default, Clone, Serialize, ToSchema)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub ws_id: i64,
    pub public: bool,
}

#[derive(Debug, Deserialize, Default, Clone, Serialize, ToSchema)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl AppState {
    pub async fn create_chat(&self, create_chat: CreateChat) -> Result<Chat, AppError> {
        self.validate_chat_members_and_name(&create_chat.members, &create_chat.name)
            .await?;
        let chat_type =
            Chat::get_chat_type_by(&create_chat.members, &create_chat.name, create_chat.public);

        let chat: Chat = sqlx::query_as(
            "INSERT INTO chats (ws_id, name, type, members) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(create_chat.ws_id)
        .bind(create_chat.name)
        .bind(chat_type)
        .bind(create_chat.members)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }

    pub async fn update_chat(&self, id: i64, update_chat: UpdateChat) -> Result<Chat, AppError> {
        if self.get_chat_by_id(id).await?.is_none() {
            return Err(AppError::NotFound("Chat not found".to_string()));
        }

        self.validate_chat_members_and_name(&update_chat.members, &update_chat.name)
            .await?;
        let chat_type =
            Chat::get_chat_type_by(&update_chat.members, &update_chat.name, update_chat.public);

        let chat: Chat = sqlx::query_as(
            "UPDATE chats SET name = $1, type = $2, members = $3 WHERE id = $4 RETURNING *",
        )
        .bind(update_chat.name)
        .bind(chat_type)
        .bind(update_chat.members)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }

    pub async fn delete_chat(&self, id: i64) -> Result<(), AppError> {
        if self.get_chat_by_id(id).await?.is_none() {
            return Err(AppError::NotFound("Chat not found".to_string()));
        }

        sqlx::query("DELETE FROM chats WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn validate_chat_members_and_name(
        &self,
        members: &[i64],
        name: &Option<String>,
    ) -> Result<(), AppError> {
        let len = members.len();
        if len < 2 {
            return Err(AppError::CreateChat(
                "At least 2 members are required".to_string(),
            ));
        }

        if len > 8 && name.is_none() {
            return Err(AppError::CreateChat(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }

        // verify if all members exist
        let users = self.fetch_chat_users_by_ids(members).await?;
        if users.len() != len {
            return Err(AppError::CreateChat(
                "Some members do not exists".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn fetch_chats_by_ws_id(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as("SELECT * FROM chats WHERE ws_id = $1")
            .bind(ws_id as i64)
            .fetch_all(&self.pool)
            .await?;
        Ok(chats)
    }

    pub async fn get_chat_by_id(&self, id: i64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
                    SELECT id, ws_id, name, type, members, created_at
                    FROM chats
                    WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat)
    }

    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let chat = sqlx::query(
            r#"
                    SELECT 1
                    FROM chats
                    WHERE id = $1 AND $2 = ANY(members)"#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat.is_some())
    }
}

#[cfg(test)]
mod tests {
    use chat_core::ChatType;

    use super::*;
    use crate::error::AppError;

    #[tokio::test]
    async fn create_chat_shourld_error() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let members_less_2_chat = CreateChat {
            name: None,
            members: vec![1],
            ws_id: 1,
            public: false,
        };
        let ret = state.create_chat(members_less_2_chat).await;
        assert!(ret.is_err());
        let err = ret.unwrap_err();
        assert_eq!(
            err.to_string(),
            "create chat error: At least 2 members are required"
        );

        let members_more_8_and_no_name_chat = CreateChat {
            name: None,
            members: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            ws_id: 1,
            public: false,
        };
        let ret = state.create_chat(members_more_8_and_no_name_chat).await;
        assert!(ret.is_err());
        let err = ret.unwrap_err();
        assert_eq!(
            err.to_string(),
            "create chat error: Group chat with more than 8 members must have a name"
        );

        let members_not_exist_chat = CreateChat {
            name: None,
            members: vec![111, 2, 3],
            ws_id: 1,
            public: false,
        };
        let ret = state.create_chat(members_not_exist_chat).await;
        assert!(ret.is_err());
        let err = ret.unwrap_err();
        assert_eq!(
            err.to_string(),
            "create chat error: Some members do not exists"
        );
        Ok(())
    }

    #[tokio::test]
    async fn create_public_chat_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let have_name_and_public_chat = CreateChat {
            name: Some("test".to_string()),
            members: vec![1, 2, 3],
            ws_id: 1,
            public: true,
        };
        let chat = state.create_chat(have_name_and_public_chat).await;
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn create_private_chat_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let have_name_and_private_chat = CreateChat {
            name: Some("test".to_string()),
            members: vec![1, 2, 3],
            ws_id: 1,
            public: false,
        };
        let chat = state.create_chat(have_name_and_private_chat).await;
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        Ok(())
    }

    #[tokio::test]
    async fn create_noname_group_chat_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let member_greater_2_and_no_name_chat = CreateChat {
            name: None,
            members: vec![1, 2, 3],
            ws_id: 1,
            public: false,
        };
        let chat = state.create_chat(member_greater_2_and_no_name_chat).await;
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Group);

        Ok(())
    }

    #[tokio::test]
    async fn create_noname_single_chat_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let member_2_and_no_name_chat = CreateChat {
            name: None,
            members: vec![1, 2],
            ws_id: 1,
            public: false,
        };
        let chat = state.create_chat(member_2_and_no_name_chat).await;
        println!("{:?}", chat);
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Single);

        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_by_ws_id_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_chats_by_ws_id(1).await?;
        assert_eq!(chats.len(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn get_by_id_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state.get_chat_by_id(1).await?;
        assert!(chat.is_some());
        let chat = chat.unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.name, Some("general".to_string()));
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.members, vec![1, 2, 3, 4, 5]);

        Ok(())
    }

    #[tokio::test]
    async fn chat_is_member_shourld_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(1, 1).await?;
        assert!(is_member);
        // user 6 is not in chat 1
        let is_member = state.is_chat_member(1, 6).await?;
        assert!(!is_member);

        // chat 10 does not exist
        let is_member = state.is_chat_member(10, 1).await?;
        assert!(!is_member);

        // chat 10 does not exist and user 10 does not exist
        let is_member = state.is_chat_member(10, 10).await?;
        assert!(!is_member);

        Ok(())
    }
}
