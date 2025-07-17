use axum::{extract::State, response::IntoResponse};
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, Encoder, HistogramVec,
    IntCounterVec, IntGauge, TextEncoder,
};
use std::{sync::Arc, time::Instant};

use crate::router::AppState;

/// Struct containing all Prometheus metrics
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppMetrics {
    // Request metrics
    pub http_requests_total: IntCounterVec,
    pub http_requests_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntGauge,
    
    // Authentication metrics
    pub auth_success_total: IntCounterVec,
    pub auth_failure_total: IntCounterVec,
    
    // Database metrics
    pub database_queries_total: IntCounterVec,
    pub database_query_duration_seconds: HistogramVec,
    
    // Application metrics
    pub template_render_duration_seconds: HistogramVec,
    pub cache_hit_total: IntCounterVec,
    pub cache_miss_total: IntCounterVec,
}

impl AppMetrics {
    #[cfg(test)]
    pub fn new_for_tests() -> Self {
        use prometheus::{IntCounterVec, IntGauge, HistogramVec};
        use prometheus::opts;
        
        // In tests, we don't register metrics with the global registry to avoid collisions
        Self {
            http_requests_total: IntCounterVec::new(
                opts!("http_requests_total", "Total number of HTTP requests"),
                &["method", "path", "status"]
            ).unwrap(),
            
            http_requests_duration_seconds: HistogramVec::new(
                prometheus::histogram_opts!("http_requests_duration_seconds", "HTTP request duration in seconds", vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
                &["method", "path"]
            ).unwrap(),
            
            http_requests_in_flight: IntGauge::new(
                "http_requests_in_flight", 
                "Number of HTTP requests currently in flight"
            ).unwrap(),
            
            auth_success_total: IntCounterVec::new(
                opts!("auth_success_total", "Total number of successful authentication attempts"),
                &["username"]
            ).unwrap(),
            
            auth_failure_total: IntCounterVec::new(
                opts!("auth_failure_total", "Total number of failed authentication attempts"),
                &["reason"]
            ).unwrap(),
            
            database_queries_total: IntCounterVec::new(
                opts!("database_queries_total", "Total number of database queries"),
                &["operation", "table"]
            ).unwrap(),
            
            database_query_duration_seconds: HistogramVec::new(
                prometheus::histogram_opts!("database_query_duration_seconds", "Database query duration in seconds", vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
                &["operation", "table"]
            ).unwrap(),
            
            template_render_duration_seconds: HistogramVec::new(
                prometheus::histogram_opts!("template_render_duration_seconds", "Template rendering duration in seconds", vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25]),
                &["template"]
            ).unwrap(),
            
            cache_hit_total: IntCounterVec::new(
                opts!("cache_hit_total", "Total number of cache hits"),
                &["cache"]
            ).unwrap(),
            
            cache_miss_total: IntCounterVec::new(
                opts!("cache_miss_total", "Total number of cache misses"),
                &["cache"]
            ).unwrap(),
        }
    }
    
    pub fn new() -> Self {
        // HTTP request metrics
        let http_requests_total = register_int_counter_vec!(
            "http_requests_total",
            "Total number of HTTP requests",
            &["method", "path", "status"]
        )
        .unwrap();

        let http_requests_duration_seconds = register_histogram_vec!(
            "http_requests_duration_seconds",
            "HTTP request duration in seconds",
            &["method", "path"],
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
        )
        .unwrap();

        let http_requests_in_flight = register_int_gauge!(
            "http_requests_in_flight",
            "Number of HTTP requests currently in flight"
        )
        .unwrap();

        // Authentication metrics
        let auth_success_total = register_int_counter_vec!(
            "auth_success_total",
            "Total number of successful authentication attempts",
            &["username"]
        )
        .unwrap();

        let auth_failure_total = register_int_counter_vec!(
            "auth_failure_total",
            "Total number of failed authentication attempts",
            &["reason"]
        )
        .unwrap();

        // Database metrics
        let database_queries_total = register_int_counter_vec!(
            "database_queries_total",
            "Total number of database queries",
            &["operation", "status"]
        )
        .unwrap();

        let database_query_duration_seconds = register_histogram_vec!(
            "database_query_duration_seconds",
            "Database query duration in seconds",
            &["operation"],
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
        )
        .unwrap();

        // Application metrics
        let template_render_duration_seconds = register_histogram_vec!(
            "template_render_duration_seconds",
            "Template rendering duration in seconds",
            &["template"],
            vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]
        )
        .unwrap();

        let cache_hit_total = register_int_counter_vec!(
            "cache_hit_total",
            "Total number of cache hits",
            &["cache"]
        )
        .unwrap();

        let cache_miss_total = register_int_counter_vec!(
            "cache_miss_total",
            "Total number of cache misses",
            &["cache"]
        )
        .unwrap();

        Self {
            http_requests_total,
            http_requests_duration_seconds,
            http_requests_in_flight,
            auth_success_total,
            auth_failure_total,
            database_queries_total,
            database_query_duration_seconds,
            template_render_duration_seconds,
            cache_hit_total,
            cache_miss_total,
        }
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware to track HTTP request metrics (request count, duration)
pub async fn track_metrics(
    State(app_state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract method and path for metric labels
    let method = req.method().to_string();
    let path = req
        .uri()
        .path()
        .to_string()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '/', "_");

    // Start timing
    let start = Instant::now();
    
    // Increment in-flight requests
    app_state.metrics.http_requests_in_flight.inc();
    
    // Process the request
    let response = next.run(req).await;
    
    // Decrement in-flight requests
    app_state.metrics.http_requests_in_flight.dec();
    
    // Calculate duration
    let duration = start.elapsed().as_secs_f64();
    
    // Record duration
    app_state
        .metrics
        .http_requests_duration_seconds
        .with_label_values(&[&method, &path])
        .observe(duration);
    
    // Record request count
    let status = response.status().as_u16().to_string();
    app_state
        .metrics
        .http_requests_total
        .with_label_values(&[&method, &path, &status])
        .inc();
    
    response
}

/// GET /metrics - Expose Prometheus metrics
pub async fn get_metrics(_state: State<Arc<AppState>>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    
    // Gather all metrics
    let metric_families = prometheus::gather();
    
    // Encode metrics into text format
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    // Return metrics as plain text with correct content type
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        buffer
    )
}

// Helper functions to track template rendering time
pub fn track_template_rendering(metrics: &AppMetrics, template_name: &str, duration: f64) {
    metrics
        .template_render_duration_seconds
        .with_label_values(&[template_name])
        .observe(duration);
}

// Helper function to track cache operations
pub fn track_cache_hit(metrics: &AppMetrics, cache_name: &str) {
    metrics
        .cache_hit_total
        .with_label_values(&[cache_name])
        .inc();
}

pub fn track_cache_miss(metrics: &AppMetrics, cache_name: &str) {
    metrics
        .cache_miss_total
        .with_label_values(&[cache_name])
        .inc();
}

// Helper functions to track database operations
pub fn track_database_query(
    metrics: &AppMetrics, 
    operation: &str, 
    status: &str, 
    duration: f64
) {
    metrics
        .database_queries_total
        .with_label_values(&[operation, status])
        .inc();
        
    metrics
        .database_query_duration_seconds
        .with_label_values(&[operation])
        .observe(duration);
}

// Helper functions to track authentication events
pub fn track_auth_success(metrics: &AppMetrics, username: &str) {
    metrics
        .auth_success_total
        .with_label_values(&[username])
        .inc();
}

pub fn track_auth_failure(metrics: &AppMetrics, reason: &str) {
    metrics
        .auth_failure_total
        .with_label_values(&[reason])
        .inc();
}
