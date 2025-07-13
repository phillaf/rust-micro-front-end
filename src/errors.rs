use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;
use tracing::error;

#[derive(Debug)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug)]
pub enum ErrorCode {
    ValidationFailed,
    UserNotFound,
    DatabaseError,
    AuthenticationFailed,
    InvalidInput,
    InternalServerError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn validation_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
    }

    pub fn user_not_found(username: impl Into<String>) -> Self {
        Self::new(ErrorCode::UserNotFound, format!("User '{}' not found", username.into()))
    }

    pub fn database_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::DatabaseError, message)
    }

    pub fn authentication_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::AuthenticationFailed, message)
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalServerError, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.code {
            ErrorCode::ValidationFailed => (StatusCode::BAD_REQUEST, "Validation failed"),
            ErrorCode::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            ErrorCode::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            ErrorCode::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            ErrorCode::InvalidInput => (StatusCode::BAD_REQUEST, "Invalid input"),
            ErrorCode::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        error!("Application error: {} - {}", error_message, self.message);
        if let Some(details) = &self.details {
            error!("Error details: {}", details);
        }

        let body = json!({
            "error": {
                "code": format!("{:?}", self.code),
                "message": self.message,
                "details": self.details
            }
        });

        (status, Json(body)).into_response()
    }
}

// Helper macro for creating errors
#[macro_export]
macro_rules! app_error {
    ($code:expr, $msg:expr) => {
        crate::errors::AppError::new($code, $msg)
    };
    ($code:expr, $msg:expr, $details:expr) => {
        crate::errors::AppError::new($code, $msg).with_details($details)
    };
}

// Conversion from common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::database_error(format!("Database operation failed: {}", err))
    }
}

impl From<minijinja::Error> for AppError {
    fn from(err: minijinja::Error) -> Self {
        AppError::internal_server_error(format!("Template rendering failed: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::invalid_input(format!("JSON parsing failed: {}", err))
    }
}
