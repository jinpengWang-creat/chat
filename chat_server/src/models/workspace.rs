use sqlx::PgPool;

use crate::error::AppError;

use super::{ChatUser, Workspace};

impl Workspace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let workspace: Workspace =
            sqlx::query_as("INSERT INTO workspaces (name, owner_id) VALUES ($1, $2) RETURNING *")
                .bind(name)
                .bind(user_id as i64)
                .fetch_one(pool)
                .await?;
        Ok(workspace)
    }

    #[allow(dead_code)]
    pub async fn update_owner(&self, owner_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let workspace: Workspace =
            sqlx::query_as("UPDATE workspaces SET owner_id = $1 WHERE id = $2 RETURNING *")
                .bind(owner_id)
                .bind(self.id)
                .fetch_one(pool)
                .await?;
        Ok(workspace)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let workspace = sqlx::query_as("SELECT * FROM workspaces WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        Ok(workspace)
    }

    pub async fn fetch_all_chat_users(
        ws_id: i64,
        pool: &PgPool,
    ) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as("SELECT id, fullname, email FROM users WHERE ws_id = $1")
            .bind(ws_id)
            .fetch_all(pool)
            .await?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{error::AppError, models::SignupUser, test_util::get_test_pool, User};

    #[tokio::test]
    async fn workspace_should_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let signup_user = SignupUser::new("test", "test@1234.com", "password");
        let user = User::create(&signup_user, &pool).await?;

        let workspace = Workspace::create("test", user.id as u64, &pool).await?;
        assert_eq!(workspace.name, "test");
        assert_eq!(workspace.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn find_by_name_should_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let workspace = Workspace::find_by_name("workspace4", &pool).await?;
        assert!(workspace.is_none());

        let workspace = Workspace::find_by_name("workspace2", &pool).await?;
        assert!(workspace.is_some());
        let workspace = workspace.unwrap();
        assert_eq!(workspace.name, "workspace2");
        Ok(())
    }

    #[tokio::test]
    async fn update_owner_should_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let workspace = Workspace::find_by_name("workspace2", &pool).await?.unwrap();
        assert_eq!(workspace.owner_id, 0);
        let workspace = workspace.update_owner(3, &pool).await?;
        assert_eq!(workspace.owner_id, 3);
        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_chat_users_should_work() -> Result<(), AppError> {
        let (_test, pool) = get_test_pool(None).await;
        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 5);
        let users = Workspace::fetch_all_chat_users(2, &pool).await?;
        assert_eq!(users.len(), 0);
        Ok(())
    }
}
