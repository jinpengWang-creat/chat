use chat_core::Workspace;

use crate::{error::AppError, state::AppState};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<Workspace, AppError> {
        let workspace: Workspace =
            sqlx::query_as("INSERT INTO workspaces (name, owner_id) VALUES ($1, $2) RETURNING *")
                .bind(name)
                .bind(user_id as i64)
                .fetch_one(&self.pool)
                .await?;
        Ok(workspace)
    }

    #[allow(dead_code)]
    pub async fn update_workspace_owner(
        &self,
        ws_id: u64,
        owner_id: u64,
    ) -> Result<Workspace, AppError> {
        let workspace: Workspace =
            sqlx::query_as("UPDATE workspaces SET owner_id = $1 WHERE id = $2 RETURNING *")
                .bind(owner_id as i64)
                .bind(ws_id as i64)
                .fetch_one(&self.pool)
                .await?;
        Ok(workspace)
    }

    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let workspace = sqlx::query_as("SELECT * FROM workspaces WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{error::AppError, models::SignupUser};

    #[tokio::test]
    async fn workspace_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let signup_user = SignupUser::new("test", "test@1234.com", "password");
        let user = state.create_user(&signup_user).await?;

        let workspace = state.create_workspace("test", user.id as u64).await?;
        assert_eq!(workspace.name, "test");
        assert_eq!(workspace.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn find_by_name_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let workspace = state.find_workspace_by_name("workspace4").await?;
        assert!(workspace.is_none());

        let workspace = state.find_workspace_by_name("workspace2").await?;
        assert!(workspace.is_some());
        let workspace = workspace.unwrap();
        assert_eq!(workspace.name, "workspace2");
        Ok(())
    }

    #[tokio::test]
    async fn update_owner_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let workspace = state.find_workspace_by_name("workspace2").await?.unwrap();
        assert_eq!(workspace.owner_id, 0);
        let workspace = state.update_workspace_owner(workspace.id as u64, 3).await?;
        assert_eq!(workspace.owner_id, 3);
        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_chat_users_should_work() -> Result<(), AppError> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let users = state.fetch_chat_users(0).await?;
        assert_eq!(users.len(), 6);
        let users = state.fetch_chat_users(1).await?;
        assert_eq!(users.len(), 0);
        Ok(())
    }
}
