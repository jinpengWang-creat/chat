use sqlx::PgPool;

use crate::error::AppError;

use super::Workspace;

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
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sqlx_db_tester::TestPg;

    use super::*;
    use crate::{error::AppError, models::SignupUser, User};

    #[tokio::test]
    async fn workspace_should_work() -> Result<(), AppError> {
        let pool = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = pool.get_pool().await;
        let signup_user = SignupUser::new("test", "test@1234.com", "password");
        let user = User::create(&signup_user, &pool).await?;

        let workspace = Workspace::create("test", user.id as u64, &pool).await?;
        assert_eq!(workspace.name, "test");
        assert_eq!(workspace.owner_id, user.id as i64);
        Ok(())
    }
}
