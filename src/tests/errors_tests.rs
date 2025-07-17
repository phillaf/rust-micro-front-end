#[cfg(test)]
mod tests {
    use crate::errors::{AppError, ErrorCode};
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn test_error_creation() {
        // Test the creation of different error types
        let validation_error = AppError::validation_failed("Invalid input");
        assert!(matches!(validation_error.code, ErrorCode::ValidationFailed));
        assert_eq!(validation_error.message, "Invalid input");

        let user_not_found = AppError::user_not_found("testuser");
        assert!(matches!(user_not_found.code, ErrorCode::UserNotFound));
        assert_eq!(user_not_found.message, "User 'testuser' not found");
        
        let db_error = AppError::database_error("Connection failed");
        assert!(matches!(db_error.code, ErrorCode::DatabaseError));
        assert_eq!(db_error.message, "Connection failed");
        
        let invalid_input = AppError::invalid_input("Invalid JSON");
        assert!(matches!(invalid_input.code, ErrorCode::InvalidInput));
        assert_eq!(invalid_input.message, "Invalid JSON");
        
        let server_error = AppError::internal_server_error("Unknown error");
        assert!(matches!(server_error.code, ErrorCode::InternalServerError));
        assert_eq!(server_error.message, "Unknown error");
    }
    
    #[test]
    fn test_error_display() {
        let error = AppError::validation_failed("Test message");
        assert_eq!(format!("{}", error), "ValidationFailed: Test message");
    }
    
    #[test]
    fn test_error_response_status_codes() {
        // Test that each error type maps to the correct HTTP status code
        let validation_error = AppError::validation_failed("Invalid input").into_response();
        assert_eq!(validation_error.status(), StatusCode::BAD_REQUEST);
        
        let not_found_error = AppError::user_not_found("testuser").into_response();
        assert_eq!(not_found_error.status(), StatusCode::NOT_FOUND);
        
        let db_error = AppError::database_error("DB error").into_response();
        assert_eq!(db_error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        let invalid_input = AppError::invalid_input("Bad input").into_response();
        assert_eq!(invalid_input.status(), StatusCode::BAD_REQUEST);
        
        let server_error = AppError::internal_server_error("Unknown error").into_response();
        assert_eq!(server_error.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    #[test]
    fn test_error_conversions() {
        // Test conversion from sqlx::Error
        let sqlx_error = sqlx::Error::PoolTimedOut;
        let app_error = AppError::from(sqlx_error);
        assert!(matches!(app_error.code, ErrorCode::DatabaseError));
        
        // Test conversion from anyhow::Error
        let anyhow_error = anyhow::anyhow!("Test error");
        let app_error = AppError::from(anyhow_error);
        assert!(matches!(app_error.code, ErrorCode::InternalServerError));
        
        // Test conversion from serde_json::Error
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let app_error = AppError::from(json_error);
        assert!(matches!(app_error.code, ErrorCode::InvalidInput));
    }
}
