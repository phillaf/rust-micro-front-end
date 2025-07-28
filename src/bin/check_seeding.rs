use anyhow::Result;
use sqlx::MySqlPool;
use std::env;

/// List of expected seed users based on 001_create_users_table.sql migration
const EXPECTED_SEED_USERS: [&str; 3] = ["admin", "testuser", "demo"];

#[derive(Debug)]
struct SeedStatus {
    is_seeded: bool,
    seed_record_count: usize,
    found_seed_users: Vec<String>,
    missing_seed_users: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get database connection string from environment variables
    let database_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        env::var("DATABASE_USERNAME").unwrap_or_else(|_| "app_user".to_string()),
        env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "secure_password".to_string()),
        env::var("DATABASE_HOST").unwrap_or_else(|_| "mysql".to_string()),
        env::var("DATABASE_PORT").unwrap_or_else(|_| "3306".to_string()),
        env::var("DATABASE_NAME").unwrap_or_else(|_| "micro_frontend".to_string()),
    );

    // Connect to the database
    let pool = MySqlPool::connect(&database_url).await?;

    // Check seeding status directly (not using the database module)
    let mut found_users = Vec::new();
    let mut missing_users = Vec::new();

    // Check for each expected seed user
    for &username in EXPECTED_SEED_USERS.iter() {
        let result = sqlx::query("SELECT username, display_name FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&pool)
            .await?;

        if result.is_some() {
            found_users.push(username.to_string());
        } else {
            missing_users.push(username.to_string());
        }
    }

    // Create seed status
    let seed_status = SeedStatus {
        is_seeded: !found_users.is_empty(),
        seed_record_count: found_users.len(),
        found_seed_users: found_users,
        missing_seed_users: missing_users,
    };

    // Output results
    println!("\n===== DATABASE SEED STATUS =====");
    println!("Database seeded: {}", if seed_status.is_seeded { "✅ YES" } else { "❌ NO" });
    println!(
        "Found {}/{} expected seed users",
        seed_status.seed_record_count,
        EXPECTED_SEED_USERS.len()
    );

    if !seed_status.found_seed_users.is_empty() {
        println!("\nFound seed users:");
        for user in &seed_status.found_seed_users {
            println!("  ✓ {}", user);
        }
    }

    if !seed_status.missing_seed_users.is_empty() {
        println!("\nMissing seed users:");
        for user in &seed_status.missing_seed_users {
            println!("  ✗ {}", user);
        }

        println!("\nTo fix missing seed data, run: just migrate");
    }

    println!("");

    // Close the pool to ensure clean shutdown
    pool.close().await;

    Ok(())
}
