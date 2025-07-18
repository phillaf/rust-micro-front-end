# Metrics Standardization for Multi-App Ecosystem

This document outlines the standardized metrics collection, storage, and visualization strategy for our micro front-end application ecosystem. It establishes consistent practices for performance monitoring, resource utilization tracking, and business metrics across all services.

## Table of Contents

1. [Metrics Architecture](#metrics-architecture)
2. [Standard Metrics](#standard-metrics)
3. [Collection Infrastructure](#collection-infrastructure)
4. [Application Setup](#application-setup)
5. [Visualization](#visualization)
6. [Alerting](#alerting)
7. [Implementation Guide](#implementation-guide)

## Metrics Architecture

Our metrics architecture follows the modern Prometheus-based approach:

```ascii
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Micro-App 1    │     │  Micro-App 2    │     │  Micro-App 3    │
│  (This App)     │     │                 │     │                 │
│  /metrics       │     │  /metrics       │     │  /metrics       │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌───────────────────────────────────────────────────────────────┐
│                        Prometheus                             │
└───────────────────────────────────┬───────────────────────────┘
                                   │
                                   │
                                   ▼
                         ┌───────────────────┐
                         │     Grafana       │
                         └─────────┬─────────┘
                                   │
                                   │
                                   ▼
                         ┌───────────────────┐
                         │   AlertManager    │
                         └───────────────────┘
```

## Standard Metrics

### Naming Conventions

All metrics must follow this naming convention:

```text
{app_name}_{metric_type}_{metric_name}{_unit_suffix}
```

For example:

- `micro_frontend_http_requests_total`
- `micro_frontend_database_query_duration_seconds`
- `micro_frontend_memory_usage_bytes`

### Required Categories

Each application must expose metrics in the following categories:

#### System Metrics

| Metric Name | Description | Type |
|-------------|-------------|------|
| `{app}_process_cpu_usage` | CPU usage percentage (0-1) | Gauge |
| `{app}_process_memory_usage_bytes` | Memory usage in bytes | Gauge |
| `{app}_process_open_fds` | Open file descriptors | Gauge |
| `{app}_process_start_time_seconds` | Process start time | Gauge |
| `{app}_process_threads` | Number of OS threads in use | Gauge |

#### HTTP Metrics

| Metric Name | Description | Type | Labels |
|-------------|-------------|------|--------|
| `{app}_http_requests_total` | Total HTTP requests | Counter | `method`, `path`, `status_code` |
| `{app}_http_request_duration_seconds` | HTTP request duration | Histogram | `method`, `path` |
| `{app}_http_request_size_bytes` | HTTP request size | Histogram | `method`, `path` |
| `{app}_http_response_size_bytes` | HTTP response size | Histogram | `method`, `path` |
| `{app}_http_in_flight_requests` | Current in-flight HTTP requests | Gauge | `method`, `path` |

#### Database Metrics

| Metric Name | Description | Type | Labels |
|-------------|-------------|------|--------|
| `{app}_database_connections` | Active DB connections | Gauge | `database`, `state` |
| `{app}_database_connection_errors_total` | DB connection errors | Counter | `database` |
| `{app}_database_queries_total` | DB query count | Counter | `database`, `operation`, `table` |
| `{app}_database_query_duration_seconds` | DB query duration | Histogram | `database`, `operation`, `table` |

#### Application-Specific Metrics

| Metric Name | Description | Type | Labels |
|-------------|-------------|------|--------|
| `{app}_auth_attempts_total` | Authentication attempts | Counter | `status` |
| `{app}_user_actions_total` | User action counts | Counter | `action`, `status` |
| `{app}_template_render_duration_seconds` | Template rendering time | Histogram | `template` |
| `{app}_cache_operations_total` | Cache operations | Counter | `operation`, `status` |
| `{app}_cache_hit_ratio` | Cache hit/miss ratio | Gauge | `cache_name` |

#### Business Metrics

| Metric Name | Description | Type | Labels |
|-------------|-------------|------|--------|
| `{app}_active_users` | Current active users | Gauge | - |
| `{app}_user_registrations_total` | Total user registrations | Counter | - |
| `{app}_feature_usage_total` | Feature usage count | Counter | `feature` |

### Labels Standardization

Standardized label names across all applications:

| Label | Values | Description |
|-------|--------|-------------|
| `environment` | `dev`, `test`, `staging`, `prod` | Deployment environment |
| `instance` | `{host}:{port}` | Instance identifier |
| `version` | `1.0.0`, etc. | Application version |
| `method` | `GET`, `POST`, etc. | HTTP method |
| `status_code` | `200`, `404`, `500`, etc. | HTTP status code |
| `path` | `/api/username`, etc. | HTTP path |
| `database` | `mysql`, `postgres`, etc. | Database type |
| `operation` | `select`, `insert`, `update`, `delete` | Database operation |
| `status` | `success`, `failure`, `timeout` | Operation status |

### Cardinality Controls

To prevent metric explosion:

- Limit dynamic labels (e.g., do not include user IDs as labels)
- For highly dynamic paths, use route templates (e.g., `/api/users/:id` instead of `/api/users/123`)
- Group similar status codes (e.g., use `2xx`, `4xx`, `5xx` for less granular metrics)

## Collection Infrastructure

### Prometheus

[Prometheus](https://prometheus.io/) is used as the central metrics collector:

- Scrapes `/metrics` endpoints on all applications
- Stores time-series data
- Provides PromQL query language
- Handles alerting rules

#### Retention Policy

- Default retention: 15 days
- Downsampled data: 180 days in Thanos or long-term storage

### Exporters

For services not directly instrumented:

- `node_exporter` for host metrics
- `mysqld_exporter` for MySQL metrics
- `blackbox_exporter` for probing endpoints
- `nginx_exporter` for NGINX metrics

## Application Setup

### Rust Metrics Implementation

Our Rust micro-frontend uses the `metrics` crate with Prometheus output:

```rust
use axum::{routing::get, Router};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use metrics::{counter, histogram, gauge};
use std::{net::SocketAddr, time::Instant};

pub fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("micro_frontend_http_request_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

pub fn metrics_middleware<B>(req: Request<B>, next: Next<B>) -> impl Future<Output = Response> {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let timer = Instant::now();
    
    // Track in-flight requests
    let in_flight = format!("micro_frontend_http_in_flight_requests{{path=\"{}\",method=\"{}\"}}", path, method);
    gauge!(in_flight, 1.0);
    
    async move {
        let response = next.run(req).await;
        
        // Decrement in-flight requests
        gauge!(in_flight, -1.0);
        
        // Record request duration
        let duration = timer.elapsed().as_secs_f64();
        histogram!(
            "micro_frontend_http_request_duration_seconds", 
            duration,
            "path" => path.clone(),
            "method" => method.to_string(),
        );
        
        // Count the request
        counter!(
            "micro_frontend_http_requests_total", 
            1,
            "path" => path,
            "method" => method.to_string(),
            "status_code" => response.status().as_u16().to_string(),
        );
        
        response
    }
}

// Add metrics endpoint to the router
pub fn metrics_route(metrics: PrometheusHandle) -> Router {
    Router::new().route("/metrics", get(move || async move { metrics.render() }))
}
```

## Visualization

### Grafana Dashboards

Standardized dashboards for all applications:

1. **Application Overview**
   - Key service health metrics
   - Error rates
   - Request volumes

2. **Performance Dashboard**
   - Response time percentiles
   - Resource utilization
   - Database performance

3. **Business Metrics**
   - User activity
   - Feature usage
   - Conversion rates

### Dashboard Templating

All dashboards must use variables for:

- Application selection
- Environment selection
- Time range selection
- Instance selection

### Example Dashboard JSON

```json
{
  "annotations": {
    "list": [
      {
        "builtIn": 1,
        "datasource": "-- Grafana --",
        "enable": true,
        "hide": true,
        "iconColor": "rgba(0, 211, 255, 1)",
        "name": "Annotations & Alerts",
        "type": "dashboard"
      },
      {
        "datasource": "Prometheus",
        "enable": true,
        "expr": "changes(micro_frontend_process_start_time_seconds{app=\"$app\", environment=\"$environment\"}[1m]) > 0",
        "name": "App Restarts",
        "showIn": 0,
        "titleFormat": "App Restart"
      }
    ]
  },
  "editable": true,
  "gnetId": null,
  "graphTooltip": 0,
  "id": 1,
  "links": [],
  "panels": [
    {
      "aliasColors": {},
      "bars": false,
      "dashLength": 10,
      "dashes": false,
      "datasource": "Prometheus",
      "fieldConfig": {
        "defaults": {
          "links": []
        },
        "overrides": []
      },
      "fill": 1,
      "fillGradient": 0,
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 0,
        "y": 0
      },
      "hiddenSeries": false,
      "id": 1,
      "legend": {
        "avg": false,
        "current": false,
        "max": false,
        "min": false,
        "show": true,
        "total": false,
        "values": false
      },
      "lines": true,
      "linewidth": 1,
      "nullPointMode": "null",
      "options": {
        "dataLinks": []
      },
      "percentage": false,
      "pointradius": 2,
      "points": false,
      "renderer": "flot",
      "seriesOverrides": [],
      "spaceLength": 10,
      "stack": false,
      "steppedLine": false,
      "targets": [
        {
          "expr": "sum(rate(micro_frontend_http_requests_total{app=\"$app\", environment=\"$environment\"}[5m])) by (status_code)",
          "interval": "",
          "legendFormat": "{{status_code}}",
          "refId": "A"
        }
      ],
      "thresholds": [],
      "timeFrom": null,
      "timeRegions": [],
      "timeShift": null,
      "title": "Request Rate by Status Code",
      "tooltip": {
        "shared": true,
        "sort": 0,
        "value_type": "individual"
      },
      "type": "graph",
      "xaxis": {
        "buckets": null,
        "mode": "time",
        "name": null,
        "show": true,
        "values": []
      },
      "yaxes": [
        {
          "format": "short",
          "label": "requests / sec",
          "logBase": 1,
          "max": null,
          "min": "0",
          "show": true
        },
        {
          "format": "short",
          "label": null,
          "logBase": 1,
          "max": null,
          "min": null,
          "show": true
        }
      ],
      "yaxis": {
        "align": false,
        "alignLevel": null
      }
    }
  ],
  "schemaVersion": 22,
  "style": "dark",
  "tags": [],
  "templating": {
    "list": [
      {
        "allValue": null,
        "current": {
          "selected": false,
          "text": "micro-frontend",
          "value": "micro-frontend"
        },
        "datasource": "Prometheus",
        "definition": "label_values(micro_frontend_process_start_time_seconds, app)",
        "hide": 0,
        "includeAll": false,
        "label": "Application",
        "multi": false,
        "name": "app",
        "options": [],
        "query": "label_values(micro_frontend_process_start_time_seconds, app)",
        "refresh": 1,
        "regex": "",
        "skipUrlSync": false,
        "sort": 0,
        "tagValuesQuery": "",
        "tags": [],
        "tagsQuery": "",
        "type": "query",
        "useTags": false
      },
      {
        "allValue": null,
        "current": {
          "selected": false,
          "text": "prod",
          "value": "prod"
        },
        "datasource": "Prometheus",
        "definition": "label_values(micro_frontend_process_start_time_seconds{app=\"$app\"}, environment)",
        "hide": 0,
        "includeAll": false,
        "label": "Environment",
        "multi": false,
        "name": "environment",
        "options": [],
        "query": "label_values(micro_frontend_process_start_time_seconds{app=\"$app\"}, environment)",
        "refresh": 1,
        "regex": "",
        "skipUrlSync": false,
        "sort": 0,
        "tagValuesQuery": "",
        "tags": [],
        "tagsQuery": "",
        "type": "query",
        "useTags": false
      }
    ]
  },
  "time": {
    "from": "now-6h",
    "to": "now"
  },
  "timepicker": {
    "refresh_intervals": [
      "5s",
      "10s",
      "30s",
      "1m",
      "5m",
      "15m",
      "30m",
      "1h",
      "2h",
      "1d"
    ]
  },
  "timezone": "",
  "title": "Micro Frontend Overview",
  "uid": "micro-frontend-overview",
  "version": 1
}
```

## Alerting

### AlertManager Configuration

Standard AlertManager configuration for all applications:

```yaml
global:
  resolve_timeout: 5m
  smtp_smarthost: 'smtp.example.com:587'
  smtp_from: 'alertmanager@example.com'
  smtp_auth_username: 'alertmanager'
  smtp_auth_password: 'password'

route:
  group_by: ['alertname', 'app', 'instance']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 12h
  receiver: 'team-emails'
  routes:
  - match:
      severity: critical
    receiver: 'pagerduty'
    repeat_interval: 4h

receivers:
- name: 'team-emails'
  email_configs:
  - to: 'team@example.com'
    send_resolved: true

- name: 'pagerduty'
  pagerduty_configs:
  - service_key: '<pagerduty-key>'
    send_resolved: true
```

### Standard Alert Rules

Required alert rules for all applications:

```yaml
groups:
- name: micro-frontend-alerts
  rules:
  - alert: HighErrorRate
    expr: |
      sum(rate(micro_frontend_http_requests_total{app="micro-frontend",status_code=~"5.."}[5m])) /
      sum(rate(micro_frontend_http_requests_total{app="micro-frontend"}[5m])) > 0.05
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High error rate detected"
      description: "Error rate is above 5% for 5 minutes (current value: {{ $value | humanizePercentage }})"

  - alert: SlowResponses
    expr: |
      histogram_quantile(0.95, sum(rate(micro_frontend_http_request_duration_seconds_bucket{app="micro-frontend"}[5m])) by (le)) > 2
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Slow response times detected"
      description: "95th percentile of response time is above 2 seconds (current value: {{ $value }}s)"

  - alert: HighMemoryUsage
    expr: |
      micro_frontend_process_memory_usage_bytes{app="micro-frontend"} / 1024 / 1024 > 500
    for: 15m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage detected"
      description: "Memory usage is above 500MB for 15 minutes (current value: {{ $value | humanizeMB }})"

  - alert: InstanceDown
    expr: |
      up{app="micro-frontend"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Instance {{ $labels.instance }} down"
      description: "{{ $labels.instance }} of app {{ $labels.app }} has been down for more than 1 minute."
```

## Implementation Guide

### Step 1: Add Metrics Dependencies

Update `Cargo.toml`:

```toml
[dependencies]
metrics = "0.21.0"
metrics-exporter-prometheus = "0.12.0"
```

### Step 2: Configure Metrics in Application

Create `src/metrics.rs`:

```rust
pub struct AppMetrics {
    prometheus_handle: PrometheusHandle,
}

impl AppMetrics {
    pub fn new() -> Self {
        let prometheus_handle = setup_metrics_recorder();
        
        // Register common metrics
        gauge!("micro_frontend_info", 1.0, "version" => env!("CARGO_PKG_VERSION"));
        
        Self {
            prometheus_handle,
        }
    }
    
    pub fn handle(&self) -> PrometheusHandle {
        self.prometheus_handle.clone()
    }
    
    // Register a database query with timing
    pub fn record_database_query(&self, operation: &str, table: &str, duration: f64, success: bool) {
        let status = if success { "success" } else { "failure" };
        
        counter!(
            "micro_frontend_database_queries_total", 
            1, 
            "operation" => operation.to_string(),
            "table" => table.to_string(),
            "status" => status.to_string()
        );
        
        histogram!(
            "micro_frontend_database_query_duration_seconds",
            duration,
            "operation" => operation.to_string(),
            "table" => table.to_string()
        );
    }
    
    // Register a template render with timing
    pub fn record_template_render(&self, template: &str, duration: f64) {
        histogram!(
            "micro_frontend_template_render_duration_seconds",
            duration,
            "template" => template.to_string()
        );
    }
    
    // Register an authentication attempt
    pub fn record_auth_attempt(&self, success: bool) {
        let status = if success { "success" } else { "failure" };
        counter!(
            "micro_frontend_auth_attempts_total", 
            1, 
            "status" => status.to_string()
        );
    }
    
    // Record current active users
    pub fn set_active_users(&self, count: u64) {
        gauge!("micro_frontend_active_users", count as f64);
    }
}
```

### Step 3: Add Metrics Endpoint to Router

Update `src/router.rs`:

```rust
use crate::metrics::AppMetrics;

pub fn create_app(db: Arc<dyn UserDatabase>, template_service: Arc<TemplateService>, metrics: Arc<AppMetrics>) -> Router {
    // Create the router
    let app = Router::new()
        .route("/health", get(get_health))
        .route("/metrics", get(|| async move { metrics.handle().render() }))
        .route("/api/username/:username", get(get_api_username))
        .route("/api/username", post(post_api_username))
        .route("/display", get(get_display_username))
        .route("/edit", get(get_edit))
        .layer(middleware_stack())
        .with_state(AppState { db, template_service, metrics });

    app
}
```

### Step 4: Instrument Key Functions

Update handlers to record metrics:

```rust
pub async fn get_display_username(
    Path(username): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let timer = Instant::now();
    let template_name = "display";
    
    // Existing code...
    
    // Record template render time
    let duration = timer.elapsed().as_secs_f64();
    state.metrics.record_template_render(template_name, duration);
    
    // Return response
    Ok(response)
}
```

### Step 5: Configure Prometheus

Create `prometheus.yml` for scraping:

```yaml
global:
  scrape_interval:     15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'rust-micro-frontend'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['app:8080']
        labels:
          app: 'micro-frontend'
          environment: 'dev'
```

### Step 6: Add Prometheus and Grafana to Docker Compose

Update `compose.yml`:

```yaml
services:
  app:
    # Existing app configuration...

  prometheus:
    image: prom/prometheus:v2.37.0
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'

  grafana:
    image: grafana/grafana:9.0.0
    depends_on:
      - prometheus
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
      - grafana-storage:/var/lib/grafana

volumes:
  grafana-storage:
```

### Step 7: Create Basic Dashboard

Save the dashboard JSON to `grafana/provisioning/dashboards/micro-frontend.json`

Create `grafana/provisioning/dashboards/dashboard.yml`:

```yaml
apiVersion: 1

providers:
  - name: 'Default'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    options:
      path: /etc/grafana/provisioning/dashboards
```

Create `grafana/provisioning/datasources/datasource.yml`:

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    orgId: 1
    url: http://prometheus:9090
    isDefault: true
    version: 1
    editable: true
```
