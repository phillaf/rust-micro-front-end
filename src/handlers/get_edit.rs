use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    Extension,
};
use minijinja::{context, Environment};
use std::sync::Arc;
use tracing::{error, info};

use crate::database::UserDatabase;

/// GET /edit - CMS component for editing display names
pub async fn get_edit(
    State(database): State<Arc<dyn UserDatabase>>,
    Extension(username): Extension<String>,
) -> Result<Html<String>, StatusCode> {
    info!("CMS request for username: {}", username);
    
    // Create template environment
    let mut env = Environment::new();
    
    // Load templates from the templates directory
    env.set_loader(minijinja::path_loader("templates"));
    
    // Get current user data to pre-populate form
    let current_display_name = match database.get_user(&username).await {
        Ok(Some(user)) => Some(user.display_name),
        Ok(None) => None,
        Err(e) => {
            error!("Database error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Load and render the edit template
    let template = env.get_template("edit.html")
        .map_err(|e| {
            error!("Failed to load edit template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let html = template.render(context! {
        username => username,
        display_name => current_display_name,
        title => format!("Edit - {}", username)
    }).map_err(|e| {
        error!("Failed to render CMS template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Html(html))
}
