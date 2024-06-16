use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};

mod chat;
mod user;
mod workspace;
pub use chat::CreateChat;
pub use user::{SigninUser, SignupUser};

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq)]
pub struct ChatUser {
    id: i64,
    fullname: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq)]
pub struct Workspace {
    id: i64,
    name: String,
    owner_id: i64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, Serialize, FromRow, Deserialize, PartialEq)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}
