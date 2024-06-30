use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chat_core::{ChatUser, User};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
    pub ws_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignupUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

impl AppState {
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, fullname, email, ws_id, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn create_user(&self, input: &SignupUser) -> Result<User, AppError> {
        // check is the email already exists
        if self.find_user_by_email(&input.email).await?.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let password_hash = hash_password(&input.password)?;
        let user: User = sqlx::query_as("INSERT INTO users (fullname, email, password_hash, ws_id) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(&input.fullname)
            .bind(&input.email)
            .bind(password_hash)
            .bind(0)
            .fetch_one(&self.pool)
            .await?;
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => {
                self.create_workspace(&input.workspace, user.id as u64)
                    .await?
            }
        };
        self.add_user_to_workspace(user.id as u64, ws.id as u64)
            .await
    }

    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        let mut user = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, ws_id, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;
        match user {
            Some(User {
                password_hash: Some(ref password_hash),
                id,
                ..
            }) => {
                if verify_password(&input.password, password_hash)? {
                    if let Some(u) = user.as_mut() {
                        u.password_hash = None;
                        if u.ws_id != input.ws_id {
                            u.ws_id = input.ws_id;
                            self.add_user_to_workspace(id as u64, input.ws_id as u64)
                                .await?;
                        }
                    }

                    Ok(user)
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub async fn add_user_to_workspace(&self, user_id: u64, ws_id: u64) -> Result<User, AppError> {
        let user = sqlx::query_as(
            "UPDATE users SET ws_id = $1 WHERE id = $2  RETURNING id, ws_id, fullname, email, created_at",
        )
        .bind(ws_id as i64)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn fetch_chat_users_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as("SELECT id, fullname, email FROM users WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    pub async fn fetch_chat_users(&self, ws_id: i64) -> Result<Vec<ChatUser>, AppError> {
        let users =
            sqlx::query_as("SELECT id, fullname, email FROM users WHERE ws_id = $1 order by id")
                .bind(ws_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(users)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(password_hash)?;
    let is_ok = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_ok)
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str, ws_id: i64) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
            ws_id,
        }
    }
}

#[cfg(test)]
impl SignupUser {
    pub fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            workspace: "default".to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn find_by_email_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let email = "jim@123.com";
        let user = state.find_user_by_email(email).await?;
        assert!(user.is_none());

        let email = "test1@none.org";
        let user = state.find_user_by_email(email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);

        Ok(())
    }

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = SignupUser::new("tom", "tom@123.com", "1qa2ws3ed");
        let user = state.create_user(&input).await?;
        assert_eq!(user.fullname, input.fullname);
        assert_eq!(user.email, input.email);
        assert!(user.id > 0);

        let user = state.find_user_by_email(&input.email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.fullname, input.fullname);
        assert_eq!(user.email, input.email);
        assert!(user.id > 0);

        let input = SigninUser::new(&input.email, &input.password, 0);
        let user = state.verify_user(&input).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn add_to_workspace_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "test3@none.org";
        let user = state.find_user_by_email(email).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.ws_id, 0);

        let ws = state.find_workspace_by_name("workspace2").await?.unwrap();
        let user = state
            .add_user_to_workspace(user.id as u64, ws.id as u64)
            .await?;
        assert_eq!(user.ws_id, ws.id);
        Ok(())
    }
}
