use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod mock;
pub mod mysql;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub display_name: String,
}

#[async_trait]
pub trait UserDatabase: Send + Sync {
    async fn get_user(&self, username: &str) -> Result<Option<User>>;
    async fn update_user_display_name(&self, username: &str, display_name: &str) -> Result<()>;
    async fn health_check(&self) -> Result<String>;
}

pub async fn create_database_adapter() -> Result<Arc<dyn UserDatabase>> {
    let adapter_type = std::env::var("DATABASE_ADAPTER")
        .unwrap_or_else(|_| "mock".to_string());
    
    match adapter_type.as_str() {
        "mock" => {
            tracing::info!("Using mock database adapter");
            Ok(Arc::new(mock::MockUserDatabase::new()))
        }
        "mysql" => {
            tracing::info!("Using MySQL database adapter");
            let mysql_adapter = mysql::MySqlUserDatabase::new().await?;
            Ok(Arc::new(mysql_adapter))
        }
        _ => {
            anyhow::bail!("Unknown database adapter: {}", adapter_type);
        }
    }
}

pub fn validate_display_name(display_name: &str) -> Result<()> {
    if display_name.is_empty() {
        anyhow::bail!("Display name cannot be empty");
    }
    
    if display_name.len() > 100 {
        anyhow::bail!("Display name cannot be longer than 100 characters");
    }
    
    // Additional validation could be added here:
    // - Check for invalid characters
    // - Check for profanity
    // - Check for specific business rules
    
    Ok(())
}

pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        anyhow::bail!("Username cannot be empty");
    }
    
    if username.len() > 50 {
        anyhow::bail!("Username cannot be longer than 50 characters");
    }
    
    // Basic alphanumeric + underscore validation
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        anyhow::bail!("Username can only contain letters, numbers, underscores, and hyphens");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_display_name_valid() {
        assert!(validate_display_name("John Doe").is_ok());
        assert!(validate_display_name("José María García-López").is_ok());
        assert!(validate_display_name("李明").is_ok());
    }

    #[test]
    fn test_validate_display_name_invalid() {
        assert!(validate_display_name("").is_err());
        assert!(validate_display_name(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test-user").is_ok());
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("").is_err());
        assert!(validate_username(&"a".repeat(51)).is_err());
        assert!(validate_username("user@domain").is_err());
        assert!(validate_username("user with spaces").is_err());
    }
}
