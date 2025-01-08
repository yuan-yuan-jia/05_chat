mod sse;

use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgListener;
use futures::StreamExt;
use sse::sse_handler;
use chat_core::{Chat, Message};
use tracing::info;

pub enum Event {
    NewChat(Chat),
    AddToChat(Chat),
    RemoveFromChat(Chat),
    NewMessage(Message)
}

const INDEX_HTML: &'static str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut listener = PgListener::connect("postgresql://postgres:postgres@localhost:5432/chat").await?;

    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();
    tokio::spawn(async move {
        while let Some(Ok(notif)) = stream.next().await {
            info!("Received notification: {:?}", notif);
        }
    });

    
    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
