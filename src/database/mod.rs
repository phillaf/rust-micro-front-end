use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

pub mod cache;
pub mod mock;
pub mod mysql;
pub mod seeding;

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

pub struct DatabaseConfig {
    pub adapter_type: String,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            adapter_type: "mock".to_string(),
            cache_enabled: false,
            cache_ttl_seconds: 300,
        }
    }
}

// Note: Removed from_env() method to support dependency injection

/// Factory function to create a database adapter based on configuration
pub async fn create_user_database(config: DatabaseConfig) -> Result<Arc<dyn UserDatabase>> {
    // Create base database adapter
    let base_adapter: Arc<dyn UserDatabase> = match config.adapter_type.as_str() {
        "mock" => {
            tracing::info!("Using mock database adapter");
            Arc::new(mock::MockUserDatabase::new())
        }
        "mysql" => {
            tracing::info!("Using MySQL database adapter");
            // Use default MySQL config for now - in a real app, we would pass MySQL-specific config here
            let mysql_adapter = mysql::MySqlUserDatabase::new_with_config(mysql::MySqlConfig::default()).await?;
            Arc::new(mysql_adapter)
        }
        _ => {
            anyhow::bail!("Unknown database adapter: {}", config.adapter_type);
        }
    };

    if config.cache_enabled {
        let cache_ttl = Duration::from_secs(config.cache_ttl_seconds);

        tracing::info!("Database caching enabled with TTL: {:?}", cache_ttl);
        Ok(Arc::new(cache::CachedUserDatabase::new(base_adapter, cache_ttl, true)))
    } else {
        tracing::info!("Database caching disabled");
        Ok(base_adapter)
    }
}

// Removed old convenience function that depended on environment variables
// Applications should now use the config module's create_database_from_env function instead
