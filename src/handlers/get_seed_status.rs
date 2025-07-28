use anyhow::Result;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::database::seeding;
use crate::router::AppState;

#[derive(Serialize)]
pub struct SeedStatusResponse {
    is_seeded: bool,
    seed_record_count: u32,
    found_seed_users: Vec<String>,
    missing_seed_users: Vec<String>,
}

/// Handler to check if the database has been seeded with initial data
pub async fn get_seed_status(State(state): State<Arc<AppState>>) -> Result<Json<SeedStatusResponse>, StatusCode> {
    match seeding::check_database_seeding(state.database.clone()).await {
        Ok(status) => {
            let response = SeedStatusResponse {
                is_seeded: status.is_seeded,
                seed_record_count: status.seed_record_count,
                found_seed_users: status.found_seed_users,
                missing_seed_users: status.missing_seed_users,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
