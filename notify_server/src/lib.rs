mod sse;
mod config;
mod error;
mod notif;

use std::ops::Deref;
use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use chat_core::middlewares::{verify_token, TokenVerify};
use chat_core::DecodingKey;
use config::AppConfig;
use dashmap::DashMap;
use error::AppError;
use notif::AppEvent;
use sse::sse_handler;
use tokio::sync::broadcast;
pub use notif::setup_pg_listener;

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    users: UserMap,
    dk: DecodingKey,
}

const INDEX_HTML: &'static str = include_str!("../index.html");

pub fn get_router() -> (Router, AppState) {
    let config = AppConfig::load().expect("Failed to load config");
    let state = AppState::new(config);
    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state.clone());
    (app, state)
}



async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<chat_core::User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load public key");
        let users = Arc::new(DashMap::new());
        Self(Arc::new(AppStateInner { config, dk, users }))
    }
}
