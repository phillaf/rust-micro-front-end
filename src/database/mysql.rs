use super::{User, UserDatabase};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

// Helper function to try to get metrics from the global metrics instance
fn try_get_metrics() -> Option<&'static crate::metrics::AppMetrics> {
    // Get metrics without risking panics
    match std::panic::catch_unwind(|| crate::router::get_metrics_instance()) {
        Ok(metrics) => metrics,
        Err(_) => {
            tracing::warn!("Failed to access metrics instance, metrics tracking will be skipped");
            None
        }
    }
}

pub struct MySqlUserDatabase {
    pool: MySqlPool,
}

/// MySQL database connection configuration
pub struct MySqlConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl Default for MySqlConfig {
    fn default() -> Self {
        Self {
            username: "app_user".to_string(),
            password: "secure_password".to_string(),
            host: "mysql".to_string(),
            port: 3306,
            database_name: "micro_frontend".to_string(),
        }
    }
}

impl MySqlUserDatabase {
    pub async fn new_with_config(config: MySqlConfig) -> Result<Self> {
        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database_name,
        );

        let pool = MySqlPool::connect(&database_url).await?;
        tracing::info!(
            "Connected to MySQL database at {}:{}/{}",
            config.host,
            config.port,
            config.database_name
        );

        Ok(Self { pool })
    }

    // Removed legacy 'new' function that has been replaced by new_with_config
    // All code should now use new_with_config instead
}

#[async_trait]
impl UserDatabase for MySqlUserDatabase {
    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let start = std::time::Instant::now();
        let operation = "get_user";

        let result = sqlx::query("SELECT username, display_name FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await;

        let duration = start.elapsed().as_secs_f64();

        // Track database operation metrics
        if let Some(metrics) = try_get_metrics() {
            let status = if result.is_ok() { "success" } else { "error" };
            crate::metrics::track_database_query(metrics, operation, status, duration);
        }

        // Process the result
        match result {
            Ok(row) => match row {
                Some(row) => {
                    let user = User {
                        username: row.get("username"),
                        display_name: row.get("display_name"),
                    };
                    Ok(Some(user))
                }
                None => Ok(None),
            },
            Err(e) => Err(e.into()),
        }
    }

    async fn update_user_display_name(&self, username: &str, display_name: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let operation = "update_user_display_name";

        let result = sqlx::query(
            "INSERT INTO users (username, display_name) VALUES (?, ?) 
             ON DUPLICATE KEY UPDATE display_name = VALUES(display_name)",
        )
        .bind(username)
        .bind(display_name)
        .execute(&self.pool)
        .await;

        let duration = start.elapsed().as_secs_f64();

        // Track database operation metrics
        if let Some(metrics) = try_get_metrics() {
            let status = if result.is_ok() { "success" } else { "error" };
            crate::metrics::track_database_query(metrics, operation, status, duration);
        }

        match result {
            Ok(_) => {
                tracing::info!("Updated display name for user '{}' in MySQL", username);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn health_check(&self) -> Result<String> {
        let start = std::time::Instant::now();
        let operation = "health_check";

        let result = sqlx::query("SELECT COUNT(*) as user_count FROM users")
            .fetch_one(&self.pool)
            .await;

        let duration = start.elapsed().as_secs_f64();

        // Track database operation metrics
        if let Some(metrics) = try_get_metrics() {
            let status = if result.is_ok() { "success" } else { "error" };
            crate::metrics::track_database_query(metrics, operation, status, duration);
        }

        match result {
            Ok(row) => {
                let user_count: i64 = row.get("user_count");
                Ok(format!("mysql_db_healthy_with_{user_count}_users"))
            }
            Err(e) => Err(e.into()),
        }
    }
}
