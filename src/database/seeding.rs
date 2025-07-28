use anyhow::Result;
use std::sync::Arc;

use crate::database::UserDatabase;

/// Marker struct to represent database seeding status
#[derive(Debug, Clone)]
pub struct SeedStatus {
    /// Whether the database contains seed data
    pub is_seeded: bool,

    /// Count of seed records found
    pub seed_record_count: u32,

    /// List of seed usernames that were found
    pub found_seed_users: Vec<String>,

    /// List of seed usernames that were expected but missing
    pub missing_seed_users: Vec<String>,
}

/// List of expected seed users based on 001_create_users_table.sql migration
pub const EXPECTED_SEED_USERS: [&str; 3] = ["admin", "testuser", "demo"];

/// Check if the database contains the expected seed data
pub async fn check_database_seeding(db: Arc<dyn UserDatabase>) -> Result<SeedStatus> {
    let mut found_users = Vec::new();
    let mut missing_users = Vec::new();

    // Check for each expected seed user
    for &username in EXPECTED_SEED_USERS.iter() {
        match db.get_user(username).await? {
            Some(_) => found_users.push(username.to_string()),
            None => missing_users.push(username.to_string()),
        }
    }

    // Database is considered seeded if we found at least one seed user
    let is_seeded = !found_users.is_empty();
    let seed_record_count = found_users.len() as u32;

    Ok(SeedStatus {
        is_seeded,
        seed_record_count,
        found_seed_users: found_users,
        missing_seed_users: missing_users,
    })
}
