use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::sync::Arc;
use tracing::info;

use crate::errors::AppError;
use crate::router::AppState;
use crate::validation::ValidatedUsername;

/// GET /display/username/{username} - Display component shows username and display name
pub async fn get_display_username(
    State(app_state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Html<String>, AppError> {
    info!("Display request for username: {}", username);
    
    // Validate username
    let validated_username = ValidatedUsername::new(username)?;
    
    // Get user data from database
    let user_data = match app_state.database.get_user(validated_username.as_str()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User not found - still render template but show error
            let html = app_state.template_service.render("display.html", context! {
                username => validated_username.as_str(),
                error => "User not found"
            })?;
            
            return Ok(Html(html));
        }
        Err(e) => {
            return Err(AppError::database_error(format!("Failed to get user: {}", e)));
        }
    };
    
    // Render the display template
    let html = app_state.template_service.render("display.html", context! {
        username => user_data.username,
        display_name => user_data.display_name,
        title => format!("Display - {}", user_data.username),
        description => format!("View the display name for user {}", user_data.username),
        keywords => "user, display, profile, username"
    })?;
    
    Ok(Html(html))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::mock::MockUserDatabase;
    use crate::metrics::AppMetrics;
    
    #[tokio::test]
    async fn test_get_display_username_success() {
        // Set up test dependencies
        let db = Arc::new(MockUserDatabase::new());
        let template_service = crate::template::TemplateService::new(false, false).unwrap();
        let app_state = Arc::new(AppState {
            database: db,
            template_service,
            metrics: AppMetrics::new_for_tests(),
        });
        
        // Call the handler with admin username
        let result = get_display_username(State(app_state), Path("admin".to_string())).await;
        
        // Check that it returns OK and contains the expected content
        assert!(result.is_ok());
        let html = result.unwrap().0;
        assert!(html.contains("Administrator"));
        assert!(html.contains("admin"));
    }
    
    #[tokio::test]
    async fn test_get_display_username_not_found() {
        // Set up test dependencies
        let db = Arc::new(MockUserDatabase::new());
        let template_service = crate::template::TemplateService::new(false, false).unwrap();
        let app_state = Arc::new(AppState {
            database: db,
            template_service,
            metrics: AppMetrics::new_for_tests(),
        });
        
        // Call the handler with a non-existent username
        let result = get_display_username(State(app_state), Path("nonexistent".to_string())).await;
        
        // Check that it returns OK (we still render the template, but with an error)
        assert!(result.is_ok());
        let html = result.unwrap().0;
        assert!(html.contains("User not found"));
    }
    
    #[tokio::test]
    async fn test_get_display_username_invalid() {
        // Set up test dependencies
        let db = Arc::new(MockUserDatabase::new());
        let template_service = crate::template::TemplateService::new(false, false).unwrap();
        let app_state = Arc::new(AppState {
            database: db,
            template_service,
            metrics: AppMetrics::new_for_tests(),
        });
        
        // Call the handler with an invalid username
        let result = get_display_username(State(app_state), Path("a".to_string())).await;
        
        // Check that it returns an error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Username must be at least"));
    }
}
