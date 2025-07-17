#[cfg(test)]
mod tests {
    use tracing::{info, error, warn};
    use tracing_subscriber::fmt::format::FmtSpan;

    #[test]
    fn test_logging_initialization() {
        // Setting up a test-only logger
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_span_events(FmtSpan::CLOSE)
            .finish();

        // This will error if there's already a global logger set,
        // but we'll ignore that since we just want to verify the code doesn't panic
        let _ = tracing::subscriber::set_global_default(subscriber);

        // Log at different levels
        info!("This is an info log");
        warn!("This is a warning log");
        error!("This is an error log");
        
        // We can't easily assert on the output since it goes to stdout/stderr,
        // but at least we can verify the code doesn't panic
    }

    #[test]
    fn test_log_structured_data() {
        // Log with structured data
        info!(
            user_id = "test123",
            action = "login",
            status = "success",
            "User logged in successfully"
        );
        
        // Again, we can't easily assert on the output,
        // but this verifies the syntax works
    }
}
