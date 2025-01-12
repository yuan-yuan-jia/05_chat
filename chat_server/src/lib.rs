mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod openapi;
use crate::handlers::{
    index_handler, list_message_handler, signin_handler, signup_handler, update_chat_handler,
};
use axum::middleware::from_fn_with_state;
use axum::routing::{get, patch, post};
use axum::Router;
pub use config::AppConfig;
use error::AppError;
use handlers::{
    create_chat_handler, delete_chat_handler, file_handler, get_chat_handler, list_chat_handler,
    list_chat_users_handler, send_message_handler, upload_handler,
};
use chat_core::{
    middlewares::{set_layer, verify_token, TokenVerify},
    DecodingKey,EncodingKey,User
};
use middlewares::verify_chat;
use sqlx::PgPool;
use std::ops::Deref;
use std::sync::Arc;
use openapi::OpenApiRouter;


#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let chat = Router::new()
        .route(
            "/:id",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/:id/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_chat))
        .route("/", get(list_chat_handler).post(create_chat_handler));

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .nest("/chats", chat)
        .route("/upload", post(upload_handler))
        .route("/files/:ws_id/*path", get(file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        // routes doesn't need token verification
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let router = Router::new()
        .openapi()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(router))
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        tokio::fs::create_dir_all(&config.server.base_dir).await?;
        let dk = DecodingKey::load(&config.auth.pk)?;
        let ek = EncodingKey::load(&config.auth.sk)?;
        let pool = PgPool::connect(&config.server.db_url).await?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        })
    }
}

impl core::fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

#[cfg(feature = "test-util")]
mod test_util {
    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
            let config = AppConfig::load()?;

            let dk = DecodingKey::load(&config.auth.pk)?;
            let ek = EncodingKey::load(&config.auth.sk)?;
            let post = config.server.db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[..post];

            let (tdb, pool) = get_test_pool(Some(server_url)).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pool,
                }),
            };
            Ok((tdb, state))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://myuser:mypassword@172.17.0.3:5432".to_string(),
        };

        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");

        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }

        ts.commit().await.expect("commit transaction failed");
        (tdb, pool)
    }
}
