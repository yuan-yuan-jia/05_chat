use axum::{extract::State, response::IntoResponse, Extension};

use crate::error::AppError;
use crate::models::User;
use crate::AppState;




pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>
) -> Result<impl IntoResponse, AppError> {
    Ok("hello")
}