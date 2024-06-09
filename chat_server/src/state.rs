use std::{fmt::Debug, ops::Deref, sync::Arc};

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
    pub async fn new_for_test(
        config: AppConfig,
    ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
        use sqlx_db_tester::TestPg;
        use std::path::Path;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("loan sk failed")?;
        let server_url = config.server.db_url.rsplit_once('/').unwrap_or_default();
        let tdb = TestPg::new(server_url.0.to_string(), Path::new("../migrations"));
        let pool = tdb.get_pool().await;
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
