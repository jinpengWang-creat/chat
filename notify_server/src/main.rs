use anyhow::Result;
use notify_server::{get_router_with_state, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let (app, state) = get_router_with_state()?;
    setup_pg_listener(state).await?;

    let addr = "0.0.0.0:6687";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
