mod config;
mod handlers;
mod error;
mod models;
mod utils;
mod middleware;
use std::ops::Deref;
use std::sync::Arc;
use axum::middleware::from_fn_with_state;
use axum::Router;
use axum::routing::{get, patch, post};
pub use config::AppConfig;
use error::AppError;
use handlers::{list_chat_handler, list_chat_users_handler};
use middleware::{set_layer, verify_token};
use sqlx::PgPool;
use crate::handlers::{index_handler, list_message_handler, sign_in_handler, sign_up_handler, update_chat_handler};
use utils::{DecodingKey, EncodingKey};

#[derive(Debug,Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: PgPool,
}



pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/chat/:id", patch(update_chat_handler).delete(update_chat_handler).post(update_chat_handler)
        )
        .route("/chat/:id/messages", get(list_message_handler))
        .route("/chat", get(list_chat_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        // routes doesn't need token verification
        .route("/signin", post(sign_in_handler))
        .route("/signup", post(sign_up_handler));


    let router = Router::new()
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
        
        let dk = DecodingKey::load(&config.auth.pk)?;
        let ek = EncodingKey::load(&config.auth.sk)?;
        let pool = PgPool::connect(&config.server.db_url).await?;
        Ok(
            Self {
                inner: Arc::new(AppStateInner { config,ek,dk,pool }),
            }
        )
    }
}


impl core::fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;


    impl AppState {
        pub async fn new_for_test(
            config: AppConfig
        ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
            use sqlx_db_tester::TestPg;
            
    
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
            None => "postgres://myuser:mypassword@172.17.0.3:5432".to_string()
        };

        let tdb = TestPg::new(
            url,
            std::path::Path::new("../migrations"),
        );
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