use axum::Json;
use axum::{extract::State, response::IntoResponse, Extension};
use chat_core::User;

use crate::error::AppError;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/chats",
    params(
    ),
    responses(
        (status = 200, description = "List of chat", body = Vec<Message>),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_chat_users(user.ws_id as _).await?;
    Ok(Json(user))
}
