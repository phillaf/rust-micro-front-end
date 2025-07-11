use anyhow::Result;
use sqlx::MySqlPool;
use std::env;

pub async fn run_migrations() -> Result<()> {
    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        env::var("DATABASE_USERNAME").unwrap_or_else(|_| "app_user".to_string()),
        env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "secure_password".to_string()),
        env::var("DATABASE_HOST").unwrap_or_else(|_| "mysql".to_string()),
        env::var("DATABASE_PORT").unwrap_or_else(|_| "3306".to_string()),
        env::var("DATABASE_NAME").unwrap_or_else(|_| "micro_frontend".to_string()),
    );

    let pool = MySqlPool::connect(&database_url).await?;
    
    tracing::info!("Running database migrations...");
    
    // Run migrations using sqlx migrate
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    tracing::info!("Database migrations completed successfully");
    
    // Close the pool to ensure clean shutdown
    pool.close().await;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    run_migrations().await?;
    
    Ok(())
}
