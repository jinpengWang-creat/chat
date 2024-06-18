use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;

use super::{Chat, ChatType, ChatUser};

#[derive(Debug, Deserialize, Default, Clone, Serialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub ws_id: i64,
    pub public: bool,
}

#[derive(Debug, Deserialize, Default, Clone, Serialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl Chat {
    pub async fn create_chat(create_chat: CreateChat, pool: &PgPool) -> Result<Self, AppError> {
        Chat::validate_members_and_name(&create_chat.members, &create_chat.name, pool).await?;
        let chat_type =
            Chat::get_chat_type_by(&create_chat.members, &create_chat.name, create_chat.public);

        let chat: Chat = sqlx::query_as(
            "INSERT INTO chats (ws_id, name, type, members) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(create_chat.ws_id)
        .bind(create_chat.name)
        .bind(chat_type)
        .bind(create_chat.members)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }

    pub async fn update_chat(
        id: i64,
        update_chat: UpdateChat,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        if !Chat::exists_by_id(id, pool).await? {
            return Err(AppError::NotFound("Chat not found".to_string()));
        }

        Chat::validate_members_and_name(&update_chat.members, &update_chat.name, pool).await?;
        let chat_type =
            Chat::get_chat_type_by(&update_chat.members, &update_chat.name, update_chat.public);

        let chat: Chat = sqlx::query_as(
            "UPDATE chats SET name = $1, type = $2, members = $3 WHERE id = $4 RETURNING *",
        )
        .bind(update_chat.name)
        .bind(chat_type)
        .bind(update_chat.members)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }

    pub async fn delete_chat(id: i64, pool: &PgPool) -> Result<(), AppError> {
        if !Chat::exists_by_id(id, pool).await? {
            return Err(AppError::NotFound("Chat not found".to_string()));
        }

        sqlx::query("DELETE FROM chats WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn validate_members_and_name(
        members: &[i64],
        name: &Option<String>,
        pool: &PgPool,
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
        let users = ChatUser::fetch_all_by_ids(members, pool).await?;
        if users.len() != len {
            return Err(AppError::CreateChat(
                "Some members do not exists".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn exists_by_id(id: i64, pool: &PgPool) -> Result<bool, AppError> {
        let chat = Chat::get_by_id(id, pool).await?;
        Ok(chat.is_some())
    }
    #[allow(dead_code)]
    pub async fn fetch_all_by_ws_id(ws_id: u64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = sqlx::query_as("SELECT * FROM chats WHERE ws_id = $1")
            .bind(ws_id as i64)
            .fetch_all(pool)
            .await?;
        Ok(chats)
    }

    #[allow(dead_code)]
    pub async fn get_by_id(id: i64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = sqlx::query_as(
            r#"
                    SELECT id, ws_id, name, type, members, created_at
                    FROM chats
                    WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(chat)
    }

    pub fn get_chat_type_by(members: &Vec<i64>, name: &Option<String>, public: bool) -> ChatType {
        let len = members.len();
        match (name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::AppError, test_util::get_test_pool};

    #[tokio::test]
    async fn create_chat_shourld_error() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let members_less_2_chat = CreateChat {
            name: None,
            members: vec![1],
            ws_id: 1,
            public: false,
        };
        let ret = Chat::create_chat(members_less_2_chat, &pool).await;
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
        let ret = Chat::create_chat(members_more_8_and_no_name_chat, &pool).await;
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
        let ret = Chat::create_chat(members_not_exist_chat, &pool).await;
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
        let (_test, pool) = get_test_pool(None).await;

        let have_name_and_public_chat = CreateChat {
            name: Some("test".to_string()),
            members: vec![1, 2, 3],
            ws_id: 1,
            public: true,
        };
        let chat = Chat::create_chat(have_name_and_public_chat, &pool).await;
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn create_private_chat_shourld_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let have_name_and_private_chat = CreateChat {
            name: Some("test".to_string()),
            members: vec![1, 2, 3],
            ws_id: 1,
            public: false,
        };
        let chat = Chat::create_chat(have_name_and_private_chat, &pool).await;
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        Ok(())
    }

    #[tokio::test]
    async fn create_noname_group_chat_shourld_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let member_greater_2_and_no_name_chat = CreateChat {
            name: None,
            members: vec![1, 2, 3],
            ws_id: 1,
            public: false,
        };
        let chat = Chat::create_chat(member_greater_2_and_no_name_chat, &pool).await;
        assert!(chat.is_ok());
        let chat = chat.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.r#type, ChatType::Group);

        Ok(())
    }

    #[tokio::test]
    async fn create_noname_single_chat_shourld_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let member_2_and_no_name_chat = CreateChat {
            name: None,
            members: vec![1, 2],
            ws_id: 1,
            public: false,
        };
        let chat = Chat::create_chat(member_2_and_no_name_chat, &pool).await;
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
        let (_test, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all_by_ws_id(1, &pool).await?;
        assert_eq!(chats.len(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn get_by_id_shourld_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let chat = Chat::get_by_id(1, &pool).await?;
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
}
