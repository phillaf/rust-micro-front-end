use super::{User, UserDatabase};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

pub struct MySqlUserDatabase {
    pool: MySqlPool,
}

impl MySqlUserDatabase {
    pub async fn new() -> Result<Self> {
        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            std::env::var("DATABASE_USERNAME").unwrap_or_else(|_| "app_user".to_string()),
            std::env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "secure_password".to_string()),
            std::env::var("DATABASE_HOST").unwrap_or_else(|_| "mysql".to_string()),
            std::env::var("DATABASE_PORT").unwrap_or_else(|_| "3306".to_string()),
            std::env::var("DATABASE_NAME").unwrap_or_else(|_| "micro_frontend".to_string()),
        );

        let pool = MySqlPool::connect(&database_url).await?;

        tracing::info!("Connected to MySQL database");

        Ok(Self { pool })
    }
}

#[async_trait]
impl UserDatabase for MySqlUserDatabase {
    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT username, display_name FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let user = User {
                    username: row.get("username"),
                    display_name: row.get("display_name"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn update_user_display_name(&self, username: &str, display_name: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (username, display_name) VALUES (?, ?) 
             ON DUPLICATE KEY UPDATE display_name = VALUES(display_name)",
        )
        .bind(username)
        .bind(display_name)
        .execute(&self.pool)
        .await?;

        tracing::info!("Updated display name for user '{}' in MySQL", username);
        Ok(())
    }

    async fn health_check(&self) -> Result<String> {
        let row = sqlx::query("SELECT COUNT(*) as user_count FROM users")
            .fetch_one(&self.pool)
            .await?;

        let user_count: i64 = row.get("user_count");
        Ok(format!("mysql_db_healthy_with_{user_count}_users"))
    }
}
