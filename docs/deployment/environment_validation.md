# Environment Variable Validation Guide

This document outlines best practices for managing and validating environment variables in the Rust Micro Front-End Application.

## Principles

1. **Fail Fast**: Application should fail at startup if required environment variables are missing or invalid
2. **Explicit Defaults**: When using defaults, they should be explicitly defined in code
3. **Granular Control**: Environment variables should be specific to individual features
4. **Secure Handling**: Sensitive values should be masked in logs and error messages
5. **Clear Validation**: Each environment variable should have clear validation rules

## Environment Variable Categories

### Required Variables

These environment variables must be set for the application to start:

- `DATABASE_ADAPTER`: Must be either `mock` or `mysql`
- `JWT_PUBLIC_KEY`: Valid PEM-encoded public key for JWT verification

### Database Configuration (when using MySQL)

These are required when `DATABASE_ADAPTER=mysql`:

- `DATABASE_HOST`: Hostname of MySQL server
- `DATABASE_NAME`: Database name
- `DATABASE_USERNAME`: Database username
- `DATABASE_PASSWORD`: Database password
- `DATABASE_PORT`: Database port (default: 3306)

### Feature Flags

These control specific application features:

- `ENABLE_METRICS`: Enable/disable Prometheus metrics (default: true)
- `ENABLE_DEBUG_LOGGING`: Enable/disable debug logging (default: false)
- `MINIFY_ENABLED`: Enable/disable HTML/CSS/JS minification (default: true)
- `ENABLE_CACHING`: Enable/disable HTTP caching headers (default: false)
- `ENABLE_DATABASE_QUERY_CACHING`: Enable/disable database query caching (default: false)
- `ENABLE_SECURITY_HEADERS`: Enable/disable security headers (default: true)

### Security Settings

- `JWT_ALGORITHM`: JWT signature algorithm (default: RS256)
- `JWT_AUDIENCE`: Expected JWT audience claim
- `JWT_ISSUER`: Expected JWT issuer claim
- `JWT_MAX_AGE_SECONDS`: Maximum token age (default: 3600)
- `JWT_CLOCK_SKEW_SECONDS`: Allowed clock skew (default: 60)

## Validation Implementation

The application uses a structured approach to validate environment variables at startup:

1. **Presence Check**: Ensure required variables exist
2. **Type Validation**: Convert string values to appropriate types
3. **Range/Format Validation**: Ensure values meet constraints
4. **Consistency Check**: Validate related variables together

Example validation flow:

```rust
// In config.rs
pub fn validate_environment() -> Result<()> {
    // 1. Check required variables
    let database_adapter = env::var("DATABASE_ADAPTER")
        .map_err(|_| anyhow!("DATABASE_ADAPTER environment variable is required"))?;
        
    // 2. Validate specific values
    match database_adapter.as_str() {
        "mock" => {},
        "mysql" => {
            // Check MySQL-specific variables when using MySQL adapter
            if env::var("DATABASE_HOST").is_err() {
                return Err(anyhow!("DATABASE_HOST is required when DATABASE_ADAPTER=mysql"));
            }
            // More MySQL variable checks...
        },
        _ => return Err(anyhow!("DATABASE_ADAPTER must be 'mock' or 'mysql'"))
    }
    
    // 3. Validate JWT settings
    let jwt_public_key = env::var("JWT_PUBLIC_KEY")
        .map_err(|_| anyhow!("JWT_PUBLIC_KEY environment variable is required"))?;
        
    // Validate PEM format
    if !jwt_public_key.contains("-----BEGIN PUBLIC KEY-----") {
        return Err(anyhow!("JWT_PUBLIC_KEY must be in PEM format"));
    }
    
    // More validation...
    Ok(())
}
```

## Production Practices

For production deployments:

1. Use a `.env` file with proper permissions (600)
2. Consider using Docker secrets for sensitive values
3. Set `ENABLE_DEBUG_LOGGING=false` to avoid exposing sensitive data
4. Validate all environment variables before container startup
5. Use distinct variables for development vs production

## Environment Variable Reference

See `.env.example` for a complete list of environment variables with descriptions and default values.

## Common Issues and Solutions

1. **Problem**: Application fails to start with "DATABASE_HOST is required" error
   **Solution**: When using `DATABASE_ADAPTER=mysql`, you must set all MySQL connection variables

2. **Problem**: JWT validation fails
   **Solution**: Verify `JWT_PUBLIC_KEY` is in proper PEM format with correct line breaks

3. **Problem**: Poor performance in production
   **Solution**: Ensure `MINIFY_ENABLED=true` and `ENABLE_CACHING=true`
