use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tracing::info;
use uuid::Uuid;

use crate::database::UserDatabase;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
    pub database_status: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthQuery {
    pub request_id: Option<String>,
}

pub async fn health_check(
    axum::extract::State(database): axum::extract::State<Arc<dyn UserDatabase>>,
    Query(params): Query<HealthQuery>
) -> Result<Json<HealthResponse>, StatusCode> {
    let request_id = params.request_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    info!("Health check requested (request_id: {})", request_id);

    let database_status = match database.health_check().await {
        Ok(status) => status,
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            format!("database_error: {}", e)
        }
    };

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        request_id,
        database_status,
    };

    Ok(Json(response))
}
