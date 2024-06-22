use anyhow::Result;
use notify_server::{get_router, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    setup_pg_listener().await?;

    let app = get_router();

    let addr = "0.0.0.0:6687";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
