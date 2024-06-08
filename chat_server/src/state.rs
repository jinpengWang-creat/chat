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

impl AppStateInner {
    async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.sk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.pk).context("loan sk failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;
        Ok(Self {
            config,
            dk,
            ek,
            pool,
        })
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        Ok(Self {
            inner: Arc::new(AppStateInner::try_new(config).await?),
        })
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
