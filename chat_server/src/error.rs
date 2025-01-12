use std::fmt::Debug;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::JWTError),
    #[error("CustomError: {0}")]
    CustomError(String),
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
    #[error("create chat error: {0}")]
    CreateChatError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("create message error: {0}")]
    CreateMessageError(String),
    #[error("{0}")]
    ChatFileError(String),
    #[error("{0}")]
    AnyhowError(#[from] anyhow::Error),
}

#[derive(Debug, serde::Serialize, ToSchema,serde::Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtError(_) => StatusCode::FORBIDDEN,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::CustomError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            &Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            &Self::CreateMessageError(_) => StatusCode::BAD_REQUEST,
            &Self::ChatFileError(_) => StatusCode::BAD_REQUEST,
            &Self::AnyhowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
