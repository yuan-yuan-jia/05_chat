mod auth;
mod chat;
mod message;


pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use message::*;


use axum::response::IntoResponse;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "index"
}