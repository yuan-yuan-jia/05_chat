use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use thiserror::Error;



#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}


impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into()
        }
    }
}

impl IntoResponse for AppError {
    
    fn into_response(self) -> axum::response::Response<axum::body::Body> {
        let status = match &self {
            AppError::IoError(_) => StatusCode::FORBIDDEN,
            AppError::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}