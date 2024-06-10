use anyhow::Result;

use tokio::net::TcpListener;
use tracing::info;

use crate::{config::AppConfig, router::get_router};

pub async fn run() -> Result<()> {
    let config = AppConfig::load()?;

    let host = &config.server.host;
    let port = &config.server.port;
    let addr = format!("{}:{}", host, port);

    let app = get_router(config).await?;

    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
