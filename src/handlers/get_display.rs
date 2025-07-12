use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use minijinja::{context, Environment};
use std::sync::Arc;
use tracing::{error, info};

use crate::database::UserDatabase;

/// GET /display/username/{username} - Display component shows username and display name
pub async fn get_display_username(
    State(database): State<Arc<dyn UserDatabase>>,
    Path(username): Path<String>,
) -> Result<Html<String>, StatusCode> {
    info!("Display request for username: {}", username);
    
    // Create template environment
    let mut env = Environment::new();
    
    // Load templates from the templates directory
    env.set_loader(minijinja::path_loader("templates"));
    
    // Get user data from database
    let user_data = match database.get_user(&username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User not found - still render template but show error
            let template = env.get_template("display.html")
                .map_err(|e| {
                    error!("Failed to load display template: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            
            let html = template.render(context! {
                username => username,
                error => "User not found"
            }).map_err(|e| {
                error!("Failed to render display template: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            
            return Ok(Html(html));
        }
        Err(e) => {
            error!("Database error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Load and render the display template
    let template = env.get_template("display.html")
        .map_err(|e| {
            error!("Failed to load display template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let html = template.render(context! {
        username => user_data.username,
        display_name => user_data.display_name,
        title => format!("Display - {}", user_data.username)
    }).map_err(|e| {
        error!("Failed to render display template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Html(html))
}
