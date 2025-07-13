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
    let adapter_type = std::env::var("DATABASE_ADAPTER").unwrap_or_else(|_| "mock".to_string());

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
