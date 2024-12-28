mod config;
mod handlers;
mod error;
mod models;

use std::ops::Deref;
use std::sync::Arc;
use axum::Router;
use axum::routing::{get, patch, post};
pub use config::AppConfig;
use crate::handlers::{index_handler, list_message_handler, sign_in_handler, sign_up_handler, update_chat_handler};

#[derive(Debug,Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}



pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);

    let api = Router::new()
        .route("/signin", post(sign_in_handler))
        .route("/signup", post(sign_up_handler))
        .route("/chat/:id", patch(update_chat_handler).delete(update_chat_handler).post(update_chat_handler)
        ).route("/chat/:id/messages", get(list_message_handler));

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state)
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


impl AppState {
    pub fn new(config: AppConfig) -> AppState {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}