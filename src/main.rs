use anyhow::Result;
use std::env;
use tokio::net::TcpListener;
use tracing::info;

mod config;
mod database;
mod errors;
mod handlers;
mod logging;
mod metrics;
mod middleware;
mod router;
mod template;
#[cfg(test)]
mod tests;
mod validation;

use config::validate_environment;
use database::create_database_adapter;
use router::create_app;
use template::create_template_service;

#[tokio::main]
async fn main() -> Result<()> {
    validate_environment()?;

    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match log_level.as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => {
                eprintln!("Invalid LOG_LEVEL: {log_level}. Using 'info' as default.");
                tracing::Level::INFO
            }
        })
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let database = create_database_adapter().await?;
    info!("- Database adapter initialized successfully");

    let template_service = create_template_service()?;
    info!("- Template service initialized successfully");

    info!("- Starting Rust Micro Front-End Application");
    info!("- Log level: {}", log_level);

    let app = create_app(database, template_service);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "80".to_string())
        .parse::<u16>()
        .unwrap_or(80);

    let bind_address = format!("0.0.0.0:{port}");
    info!("Server binding to {}", bind_address);

    let listener = TcpListener::bind(&bind_address).await?;

    info!("Server started successfully on http://{}", bind_address);
    info!("Health check available at http://{}/health", bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}
