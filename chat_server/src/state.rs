use std::{fmt::Debug, fs, ops::Deref, sync::Arc};

use anyhow::{Context, Result};

use sqlx::PgPool;

use crate::{
    config::AppConfig,
    error::AppError,
    utils::{DecodingKey, EncodingKey},
};

#[derive(Clone, Debug)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub config: AppConfig,
    pub dk: DecodingKey,
    pub ek: EncodingKey,
    pub pool: PgPool,
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir).context("create base dir failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("loan sk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pool,
            }),
        })
    }
}

#[cfg(test)]
impl AppState {
    pub async fn new_for_test() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx::Executor;
        use sqlx_db_tester::TestPg;
        use std::path::Path;
        let config = AppConfig::load()?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("loan sk failed")?;
        let server_url = config.server.db_url.rsplit_once('/').unwrap_or_default();
        let tdb = TestPg::new(server_url.0.to_string(), Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut tx = pool.begin().await.expect("begin failed");
        for q in sql {
            if q.trim().is_empty() {
                continue;
            }
            tx.execute(q).await.expect("execute failed");
        }
        tx.commit().await.expect("commit failed");
        let state = Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pool,
            }),
        };
        Ok((tdb, state))
    }
}
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_db_url() {
        let config = AppConfig::load().unwrap();
        let server_url = config.server.db_url.rsplit_once('/').unwrap_or_default();
        assert_eq!(server_url.0, "postgres://postgres:postgres@localhost:5432");
        assert_eq!(server_url.1, "chat");
    }
}
