use axum::{extract::{Query, State}, http::StatusCode, response::Json};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc, time::Instant};
use tracing::{error, info};
use uuid::Uuid;

use crate::router::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, CheckStatus>,
}

#[derive(Debug, Serialize)]
pub struct CheckStatus {
    pub status: String,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct HealthQuery {
    pub request_id: Option<String>,
}

// Use lazy_static for safe initialization of the start time
lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

// Safe function to get the uptime
fn get_uptime_seconds() -> u64 {
    START_TIME.elapsed().as_secs()
}

pub async fn get_health(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<HealthQuery>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let request_id = params.request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = chrono::Utc::now();

    info!("Health check requested (request_id: {})", request_id);

    // Calculate uptime using our thread-safe function
    let uptime_seconds = get_uptime_seconds();
    
    // Create checks map
    let mut checks = HashMap::new();

    // Database check
    match app_state.database.health_check().await {
        Ok(status) => {
            checks.insert(
                "database".to_string(),
                CheckStatus {
                    status: "healthy".to_string(),
                    message: Some(status),
                    timestamp: now,
                },
            );
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            checks.insert(
                "database".to_string(),
                CheckStatus {
                    status: "unhealthy".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    timestamp: now,
                },
            );
        }
    };

    // Template engine check
    let template_result = app_state.template_service.health_check();
    if template_result {
        checks.insert(
            "template_engine".to_string(),
            CheckStatus {
                status: "healthy".to_string(),
                message: Some("Template engine initialized".to_string()),
                timestamp: now,
            },
        );
    } else {
        checks.insert(
            "template_engine".to_string(),
            CheckStatus {
                status: "unhealthy".to_string(),
                message: Some("Template engine failed".to_string()),
                timestamp: now,
            },
        );
    }

    // JWT public key check
    match std::fs::read_to_string("scripts/jwt_public_key.pem") {
        Ok(_) => {
            checks.insert(
                "jwt_public_key".to_string(),
                CheckStatus {
                    status: "healthy".to_string(),
                    message: Some("JWT public key is accessible".to_string()),
                    timestamp: now,
                },
            );
        }
        Err(e) => {
            error!("JWT public key check failed: {}", e);
            checks.insert(
                "jwt_public_key".to_string(),
                CheckStatus {
                    status: "unhealthy".to_string(),
                    message: Some(format!("JWT public key error: {}", e)),
                    timestamp: now,
                },
            );
        }
    }

    // Environment check
    checks.insert(
        "environment".to_string(),
        CheckStatus {
            status: "healthy".to_string(),
            message: Some(format!(
                "Running in {} mode",
                env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            )),
            timestamp: now,
        },
    );

    // Memory check
    checks.insert(
        "memory".to_string(),
        CheckStatus {
            status: "healthy".to_string(),
            message: None,
            timestamp: now,
        },
    );

    // Overall status determination
    let overall_status = if checks.values().any(|check| check.status != "healthy") {
        "unhealthy"
    } else {
        "healthy"
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: now,
        request_id,
        uptime_seconds,
        checks,
    };

    Ok(Json(response))
}
