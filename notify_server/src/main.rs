use axum::ServiceExt;
use notify_server::{get_router, setup_pg_listener};
use tokio::net::TcpListener;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let addr = "0.0.0.0:6687";
    let app = get_router();
    setup_pg_listener().await?;
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
