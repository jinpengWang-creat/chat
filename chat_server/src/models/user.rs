use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{error::AppError, User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegister {
    pub fullname: String,
    pub email: String,
    pub password: String,
}

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user =
            sqlx::query_as("SELECT id, fullname, email, created_at FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }

    pub async fn create_user(input: &UserRegister, pool: &PgPool) -> Result<Self, AppError> {
        let password_hash = hash_password(&input.password)?;

        let user = sqlx::query_as("INSERT INTO users (fullname, email, password_hash) VALUES ($1, $2, $3) RETURNING id, fullname, email, created_at")
            .bind(&input.fullname)
            .bind(&input.email)
            .bind(password_hash)
            .fetch_one(pool)
            .await?;
        Ok(user)
    }

    pub async fn verify(input: &UserLogin, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let mut user = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
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
                    }
                    Ok(user)
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
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
impl UserLogin {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl UserRegister {
    pub fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }

    // use sqlx_db_tester::TestPg;
    // use std::path::Path;
    // #[tokio::test]
    // async fn create_and_verify_user_should_work() -> Result<()> {
    //     let pool = TestPg::new(
    //         "postgres://postgres:postgres@localhost:5432".to_string(),
    //         Path::new("../migrations"),
    //     );
    //     let pool = pool.get_pool().await;
    //     let input = UserRegister::new("tom", "tom@123.com", "1qa2ws3ed");
    //     let user = User::create_user(&input, &pool).await?;
    //     assert_eq!(user.fullname, input.fullname);
    //     assert_eq!(user.email, input.email);
    //     assert!(user.id > 0);

    //     let user = User::find_by_email(&input.email, &pool).await?;
    //     assert!(user.is_some());
    //     let user = user.unwrap();
    //     assert_eq!(user.fullname, input.fullname);
    //     assert_eq!(user.email, input.email);
    //     assert!(user.id > 0);

    //     let input = UserLogin::new(&input.email, &input.password);
    //     let user = User::verify(&input, &pool).await?;
    //     assert!(user.is_some());

    //     Ok(())
    // }
}
