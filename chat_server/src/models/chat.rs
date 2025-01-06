use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{error::AppError, AppState};

use super::{Chat, ChatType, ChatUser};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_chat(
        &self,
        input: CreateChat,
        ws_id: u64,
    ) -> Result<Chat, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(
                AppError::CreateChatError("Chat must have at least 2 members".to_string()),
            );
        }

        if len > 8 && input.name.is_none() {
            return Err(
                AppError::CreateChatError("Group chat must have a name".to_string()),
            );
        }

        // verify if all members exits
        let users = self.fetch_chat_user_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                    "Some members do not exits".to_string(),
                      )
           );
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
           }  
        };

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }


    pub async fn fetch_chats(
        &self,
        ws_id: u64,
    ) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
             SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1 
            "#
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn get_chat_by_id(
        &self,
        id: u64,
    ) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };

        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::test_util::get_test_pool;
    use super::*;
    use anyhow::{Ok, Result};

    #[tokio::test]
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = state.create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
        Ok(())
    }
    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()>{
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("general", &[1, 2, 3], true);
        let chat = state.create_chat(input, 1)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }
    #[tokio::test]
    async fn chat_get_by_id_should_work() -> Result<()>{
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state.get_chat_by_id(1)
            .await
            .expect("get chat by id failed")
            .unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.unwrap(), "general");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);

        Ok(())
    }
    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_chats(1)
            .await
            .expect("fetch all chats failed");
        assert_eq!(chats.len(), 4);

        Ok(())
    }
}