use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    chat::run().await?;
    Ok(())
}
