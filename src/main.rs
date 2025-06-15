use anyhow::Result;
use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct HealthQuery {
    request_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing based on LOG_LEVEL environment variable
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match log_level.as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => {
                eprintln!("Invalid LOG_LEVEL: {}. Using 'info' as default.", log_level);
                tracing::Level::INFO
            }
        })
        .with_target(false)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;

    // Validate required environment variables
    validate_environment()?;

    info!("🚀 Starting Rust Micro Front-End Application");
    info!("📊 Log level: {}", log_level);

    // Build the application routes
    let app = create_app();

    // Determine the port to bind to
    let port = env::var("PORT")
        .unwrap_or_else(|_| "80".to_string())
        .parse::<u16>()
        .unwrap_or(80);
    
    let bind_address = format!("0.0.0.0:{}", port);
    info!("🌐 Server binding to {}", bind_address);

    // Create the TCP listener
    let listener = TcpListener::bind(&bind_address).await?;
    
    info!("✅ Server started successfully on http://{}", bind_address);
    info!("🏥 Health check available at http://{}/health", bind_address);

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
}

async fn health_check(Query(params): Query<HealthQuery>) -> Result<Json<HealthResponse>, StatusCode> {
    let request_id = params.request_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    info!("🏥 Health check requested (request_id: {})", request_id);

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        request_id,
    };

    Ok(Json(response))
}

fn validate_environment() -> Result<()> {
    info!("🔍 Validating environment variables...");

    // Required environment variables
    let required_vars = vec![
        "DATABASE_ADAPTER",
        "JWT_PUBLIC_KEY",
    ];

    let mut missing_vars = Vec::new();
    let mut validation_errors = Vec::new();

    // Check for missing required variables
    for var in required_vars {
        if env::var(var).is_err() {
            missing_vars.push(var);
        }
    }

    if !missing_vars.is_empty() {
        anyhow::bail!("Missing required environment variables: {}", missing_vars.join(", "));
    }

    // Validate DATABASE_ADAPTER
    let database_adapter = env::var("DATABASE_ADAPTER")?;
    if !["mock", "mysql"].contains(&database_adapter.as_str()) {
        validation_errors.push(format!("DATABASE_ADAPTER must be 'mock' or 'mysql', got: {}", database_adapter));
    } else {
        info!("📊 Database adapter: {}", database_adapter);
    }

    // Validate LOG_LEVEL if provided
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        if !["trace", "debug", "info", "warn", "error"].contains(&log_level.as_str()) {
            validation_errors.push(format!("LOG_LEVEL must be one of: trace, debug, info, warn, error. Got: {}", log_level));
        }
    }

    // Validate boolean flags
    let boolean_flags = vec![
        "ENABLE_METRICS",
        "ENABLE_DEBUG_LOGGING", 
        "ENABLE_MINIFICATION",
        "ENABLE_CACHING",
        "ENABLE_SECURITY_HEADERS",
        "ENABLE_RATE_LIMITING",
        "ENABLE_TEMPLATE_CACHING",
        "ENABLE_DATABASE_QUERY_CACHING",
        "ENABLE_GZIP_COMPRESSION",
        "ENABLE_BROTLI_COMPRESSION",
    ];

    for flag in boolean_flags {
        if let Ok(value) = env::var(flag) {
            if !["true", "false"].contains(&value.as_str()) {
                validation_errors.push(format!("{} must be 'true' or 'false', got: {}", flag, value));
            }
        }
    }

    // Validate numeric values
    if let Ok(port_str) = env::var("DATABASE_PORT") {
        match port_str.parse::<u16>() {
            Ok(port) if port > 0 => {
                info!("📊 Database port: {}", port);
            },
            _ => validation_errors.push(format!("DATABASE_PORT must be a valid port number (1-65535), got: {}", port_str)),
        }
    }

    if !validation_errors.is_empty() {
        anyhow::bail!("Environment validation errors:\n{}", validation_errors.join("\n"));
    }

    // Log successful validation and key configuration
    info!("✅ Environment validation completed successfully");
    
    // Log key configuration (non-sensitive values only)
    if let Ok(debug_logging) = env::var("ENABLE_DEBUG_LOGGING") {
        if debug_logging == "true" {
            info!("🐛 Debug logging enabled");
        }
    }
    
    if let Ok(metrics) = env::var("ENABLE_METRICS") {
        if metrics == "true" {
            info!("📈 Metrics collection enabled");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_environment_validation_success() {
        // Set required environment variables for test
        env::set_var("DATABASE_ADAPTER", "mock");
        env::set_var("JWT_PUBLIC_KEY", "test-key");
        
        let result = validate_environment();
        assert!(result.is_ok());
        
        // Clean up
        env::remove_var("DATABASE_ADAPTER");
        env::remove_var("JWT_PUBLIC_KEY");
    }

    #[test]
    fn test_environment_validation_missing_required() {
        // Remove required variables if they exist
        env::remove_var("DATABASE_ADAPTER");
        env::remove_var("JWT_PUBLIC_KEY");
        
        let result = validate_environment();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required environment variables"));
    }

    #[test]
    fn test_environment_validation_invalid_database_adapter() {
        env::set_var("DATABASE_ADAPTER", "invalid");
        env::set_var("JWT_PUBLIC_KEY", "test-key");
        
        let result = validate_environment();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DATABASE_ADAPTER must be"));
        
        // Clean up
        env::remove_var("DATABASE_ADAPTER");
        env::remove_var("JWT_PUBLIC_KEY");
    }
}
