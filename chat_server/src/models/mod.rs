use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

mod user;

#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct User {
    id: i64,
    fullname: String,
    email: String,
    #[sqlx(default)]
    #[serde(skip)]
    password_hash: Option<String>,
    created_at: DateTime<Utc>,
}
