use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq, ToSchema)]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub ws_id: i64,
}

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq, ToSchema)]
pub struct ChatUser {
    id: i64,
    fullname: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq, ToSchema)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq, ToSchema)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq, ToSchema)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i64, fullname: &str, email: &str, ws_id: i64) -> Self {
        Self {
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
            ws_id,
        }
    }
}

impl Chat {
    pub fn get_chat_type_by(members: &[i64], name: &Option<String>, public: bool) -> ChatType {
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
