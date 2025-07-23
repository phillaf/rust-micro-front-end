use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

pub mod cache;
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
    let adapter_type = std::env::var("DATABASE_ADAPTER").unwrap_or_else(|_| "mock".to_string());

    // Create base database adapter
    let base_adapter: Arc<dyn UserDatabase> = match adapter_type.as_str() {
        "mock" => {
            tracing::info!("Using mock database adapter");
            Arc::new(mock::MockUserDatabase::new())
        }
        "mysql" => {
            tracing::info!("Using MySQL database adapter");
            let mysql_adapter = mysql::MySqlUserDatabase::new().await?;
            Arc::new(mysql_adapter)
        }
        _ => {
            anyhow::bail!("Unknown database adapter: {}", adapter_type);
        }
    };

    // Check if caching is enabled
    let cache_enabled = std::env::var("ENABLE_DATABASE_QUERY_CACHING")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if cache_enabled {
        // Parse cache TTL from environment
        let cache_ttl_seconds = std::env::var("DATABASE_CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);

        let cache_ttl = Duration::from_secs(cache_ttl_seconds);

        tracing::info!("Database caching enabled with TTL: {:?}", cache_ttl);
        Ok(Arc::new(cache::CachedUserDatabase::new(base_adapter, cache_ttl, true)))
    } else {
        tracing::info!("Database caching disabled");
        Ok(base_adapter)
    }
}
