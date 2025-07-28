use anyhow::Result;
use std::env;
use tracing::info;

pub fn validate_environment() -> Result<()> {
    info!("Validating environment variables...");

    let required_vars = vec!["DATABASE_ADAPTER", "JWT_PUBLIC_KEY"];

    let mut missing_vars = Vec::new();
    let mut validation_errors = Vec::new();

    for var in required_vars {
        if env::var(var).is_err() {
            missing_vars.push(var);
        }
    }

    if !missing_vars.is_empty() {
        anyhow::bail!("Missing required environment variables: {}", missing_vars.join(", "));
    }

    let database_adapter = env::var("DATABASE_ADAPTER")?;
    if !["mock", "mysql"].contains(&database_adapter.as_str()) {
        validation_errors.push(format!("DATABASE_ADAPTER must be 'mock' or 'mysql', got: {database_adapter}"));
    } else {
        info!("Database adapter: {}", database_adapter);
    }

    if let Ok(log_level) = env::var("LOG_LEVEL") {
        if !["trace", "debug", "info", "warn", "error"].contains(&log_level.as_str()) {
            validation_errors.push(format!(
                "LOG_LEVEL must be one of: trace, debug, info, warn, error. Got: {log_level}"
            ));
        }
    }

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
                validation_errors.push(format!("{flag} must be 'true' or 'false', got: {value}"));
            }
        }
    }

    if let Ok(port_str) = env::var("DATABASE_PORT") {
        match port_str.parse::<u16>() {
            Ok(port) if port > 0 => {
                info!("Database port: {}", port);
            }
            _ => {
                validation_errors.push(format!("DATABASE_PORT must be a valid port number (1-65535), got: {port_str}"))
            }
        }
    }

    if !validation_errors.is_empty() {
        anyhow::bail!("Environment validation errors:\n{}", validation_errors.join("\n"));
    }

    info!("Environment validation completed successfully");

    if let Ok(debug_logging) = env::var("ENABLE_DEBUG_LOGGING") {
        if debug_logging == "true" {
            info!("Debug logging enabled");
        }
    }

    if let Ok(metrics) = env::var("ENABLE_METRICS") {
        if metrics == "true" {
            info!("Metrics collection enabled");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Use a mutex to prevent tests from running environment validation concurrently
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_environment_validation_success() {
        // Lock environment for this test
        let _lock = ENV_MUTEX.lock().unwrap();

        // Save original values
        let original_db_adapter = env::var("DATABASE_ADAPTER").ok();
        let original_jwt_key = env::var("JWT_PUBLIC_KEY").ok();

        // Set test values
        env::set_var("DATABASE_ADAPTER", "mock");
        env::set_var("JWT_PUBLIC_KEY", "test-key");

        let result = validate_environment();
        assert!(result.is_ok());

        // Restore original values
        if let Some(value) = original_db_adapter {
            env::set_var("DATABASE_ADAPTER", value);
        } else {
            env::remove_var("DATABASE_ADAPTER");
        }

        if let Some(value) = original_jwt_key {
            env::set_var("JWT_PUBLIC_KEY", value);
        } else {
            env::remove_var("JWT_PUBLIC_KEY");
        }
    }

    #[test]
    fn test_environment_validation_missing_required() {
        // Lock environment for this test
        let _lock = ENV_MUTEX.lock().unwrap();

        // Save original values
        let original_db_adapter = env::var("DATABASE_ADAPTER").ok();
        let original_jwt_key = env::var("JWT_PUBLIC_KEY").ok();

        // Remove required environment variables
        env::remove_var("DATABASE_ADAPTER");
        env::remove_var("JWT_PUBLIC_KEY");

        let result = validate_environment();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing required environment variables"));
        assert!(error_msg.contains("DATABASE_ADAPTER"));
        assert!(error_msg.contains("JWT_PUBLIC_KEY"));

        // Restore original values
        if let Some(value) = original_db_adapter {
            env::set_var("DATABASE_ADAPTER", value);
        }

        if let Some(value) = original_jwt_key {
            env::set_var("JWT_PUBLIC_KEY", value);
        }
    }

    #[test]
    fn test_environment_validation_invalid_database_adapter() {
        // Lock environment for this test
        let _lock = ENV_MUTEX.lock().unwrap();

        // Save original values
        let original_db_adapter = env::var("DATABASE_ADAPTER").ok();
        let original_jwt_key = env::var("JWT_PUBLIC_KEY").ok();

        // Set test values
        env::set_var("DATABASE_ADAPTER", "invalid");
        env::set_var("JWT_PUBLIC_KEY", "test-key");

        let result = validate_environment();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("DATABASE_ADAPTER must be"));

        // Restore original values
        if let Some(value) = original_db_adapter {
            env::set_var("DATABASE_ADAPTER", value);
        } else {
            env::remove_var("DATABASE_ADAPTER");
        }

        if let Some(value) = original_jwt_key {
            env::set_var("JWT_PUBLIC_KEY", value);
        } else {
            env::remove_var("JWT_PUBLIC_KEY");
        }
    }
}
