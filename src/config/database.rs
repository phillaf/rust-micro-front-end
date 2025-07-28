use anyhow::Result;
use std::{env, sync::Arc, time::Duration};

use crate::database::{create_user_database, mysql, DatabaseConfig, UserDatabase};

/// Load database configuration from environment variables
pub fn load_database_config() -> DatabaseConfig {
    let adapter_type = env::var("DATABASE_ADAPTER").unwrap_or_else(|_| "mock".to_string());

    let cache_enabled = env::var("ENABLE_DATABASE_QUERY_CACHING")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    let cache_ttl_seconds = env::var("DATABASE_CACHE_TTL_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    DatabaseConfig {
        adapter_type,
        cache_enabled,
        cache_ttl_seconds,
    }
}

/// Load MySQL-specific configuration from environment variables
pub fn load_mysql_config() -> mysql::MySqlConfig {
    mysql::MySqlConfig {
        username: env::var("DATABASE_USERNAME").unwrap_or_else(|_| "app_user".to_string()),
        password: env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "secure_password".to_string()),
        host: env::var("DATABASE_HOST").unwrap_or_else(|_| "mysql".to_string()),
        port: env::var("DATABASE_PORT")
            .unwrap_or_else(|_| "3306".to_string())
            .parse()
            .unwrap_or(3306),
        database_name: env::var("DATABASE_NAME").unwrap_or_else(|_| "micro_frontend".to_string()),
    }
}

/// Create a database instance with configuration from environment variables
pub async fn create_database_from_env() -> Result<Arc<dyn UserDatabase>> {
    let config = load_database_config();

    // Special handling for MySQL adapter to pass specific MySQL config
    if config.adapter_type == "mysql" {
        let mysql_config = load_mysql_config();
        let base_adapter = Arc::new(mysql::MySqlUserDatabase::new_with_config(mysql_config).await?);

        if config.cache_enabled {
            let cache_ttl = Duration::from_secs(config.cache_ttl_seconds);
            Ok(Arc::new(crate::database::cache::CachedUserDatabase::new(
                base_adapter,
                cache_ttl,
                true,
            )))
        } else {
            Ok(base_adapter)
        }
    } else {
        // For non-MySQL adapters, use the factory
        create_user_database(config).await
    }
}
