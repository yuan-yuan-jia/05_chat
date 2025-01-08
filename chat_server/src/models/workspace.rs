use chat_core::Workspace;
use sqlx::PgPool;

use crate::{error::AppError, AppState};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: i32) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
               INSERT INTO workspaces (name, owner_id)
                VALUES ($1, $2)
               RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn update_workspace_owner(
        &self,
        id: u64,
        owner_id: u64,
    ) -> Result<Workspace, AppError> {
        // update owner_id in two cases 1) owner_id = 0 2) owner's ws_id = id
        let ws = sqlx::query_as(
            r#"
        UPDATE workspaces
        SET owner_id = $1
        WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at
        "#,
        )
        .bind(owner_id as i64)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;

        
        Ok(ws)

    }
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
    #[allow(dead_code)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE id = $1
        "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }
    // #[allow(dead_code)]
    // pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
    //     let users = sqlx::query_as(
    //         r#"
    //     SELECT id, fullname, email
    //     FROM users
    //     WHERE ws_id = $1 order by id
    //     "#,
    //     )
    //     .bind(id as i64)
    //     .fetch_all(pool)
    //     .await?;
    //     Ok(users)
    // }
}


