use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{error::AppError, User};

use super::{ChatUser, Workspace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
    pub ws_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupUser {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, fullname, email, ws_id, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn create(input: &SignupUser, pool: &PgPool) -> Result<Self, AppError> {
        // check is the email already exists
        if Self::find_by_email(&input.email, pool).await?.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        let password_hash = hash_password(&input.password)?;
        let user: User = sqlx::query_as("INSERT INTO users (fullname, email, password_hash, ws_id) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(&input.fullname)
            .bind(&input.email)
            .bind(password_hash)
            .bind(0)
            .fetch_one(pool)
            .await?;
        let ws = match Workspace::find_by_name(&input.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&input.workspace, user.id as u64, pool).await?,
        };
        user.add_to_workspace(ws.id, pool).await
    }

    pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let mut user = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, ws_id, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(pool)
        .await?;
        match user {
            Some(User {
                password_hash: Some(ref password_hash),
                ..
            }) => {
                if verify_password(&input.password, password_hash)? {
                    if let Some(u) = user.as_mut() {
                        u.password_hash = None;
                        if u.ws_id != input.ws_id {
                            u.ws_id = input.ws_id;
                            u.add_to_workspace(input.ws_id, pool).await?;
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

    pub async fn add_to_workspace(&self, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let user = sqlx::query_as(
            "UPDATE users SET ws_id = $1 WHERE id = $2  RETURNING id, ws_id, fullname, email, created_at",
        )
        .bind(ws_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }
}

impl ChatUser {
    pub async fn fetch_all_by_ids(ids: &[i64], pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as("SELECT id, fullname, email FROM users WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
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

#[cfg(test)]
mod tests {

    use crate::test_util::get_test_pool;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn find_by_email_should_work() -> Result<()> {
        let (_test, pool) = get_test_pool(None).await;

        let email = "jim@123.com";
        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_none());

        let email = "user1@123.com";
        let user = User::find_by_email(email, &pool).await?;
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
        let (_test, pool) = get_test_pool(None).await;
        let input = SignupUser::new("tom", "tom@123.com", "1qa2ws3ed");
        let user = User::create(&input, &pool).await?;
        assert_eq!(user.fullname, input.fullname);
        assert_eq!(user.email, input.email);
        assert!(user.id > 0);

        let user = User::find_by_email(&input.email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.fullname, input.fullname);
        assert_eq!(user.email, input.email);
        assert!(user.id > 0);

        let input = SigninUser::new(&input.email, &input.password, 0);
        let user = User::verify(&input, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn add_to_workspace_should_work() -> Result<()> {
        let (_test, pool) = get_test_pool(None).await;

        let email = "user3@123.com";
        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.ws_id, 1);

        let ws = Workspace::find_by_name("workspace2", &pool).await?.unwrap();
        let user = user.add_to_workspace(ws.id, &pool).await?;
        assert_eq!(user.ws_id, ws.id);
        Ok(())
    }
}
