mod sse;

use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use sse::sse_handler;

const INDEX_HTML: &'static str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
