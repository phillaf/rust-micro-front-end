# Performance Tuning Guide

This guide provides strategies and best practices for optimizing the performance of the Rust Micro Front-End Application.

## Table of Contents

1. [Key Performance Metrics](#key-performance-metrics)
2. [Server Optimization](#server-optimization)
3. [Database Optimization](#database-optimization)
4. [Front-End Performance](#front-end-performance)
5. [Caching Strategies](#caching-strategies)
6. [Performance Testing](#performance-testing)
7. [Production Optimization](#production-optimization)

## Key Performance Metrics

The application aims to achieve the following performance targets:

| Metric | Target | Description |
|--------|--------|-------------|
| First Contentful Paint (FCP) | < 1.2s | Time until first content appears |
| Largest Contentful Paint (LCP) | < 2.5s | Time until main content renders |
| Time to Interactive (TTI) | < 3.8s | Time until page becomes interactive |
| Cumulative Layout Shift (CLS) | < 0.1 | Measure of visual stability |
| Server Response Time | < 200ms | Time to first byte from server |
| Database Query Time | < 100ms | Maximum time for typical queries |

### Monitoring Performance

Use the built-in metrics endpoint to monitor performance:

```bash
curl http://localhost:3000/metrics
```

Key metrics to monitor include:

- `http_requests_duration_seconds`: HTTP request processing time
- `http_requests_total`: HTTP request count by status code
- `database_query_duration_seconds`: Database query execution time
- `template_render_duration_seconds`: Template rendering time

## Server Optimization

### 1. Worker Configuration

The application uses Tokio's multi-threaded runtime. Optimize the number of worker threads based on your server's CPU cores:

```rust
// In main.rs
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get())  // Use all available cores
    .enable_all()
    .build()
    .unwrap();
```

### 2. Connection Pooling

Fine-tune the database connection pool based on your workload:

```bash
# Recommended environment variables
DATABASE_MAX_CONNECTIONS=25     # Adjust based on CPU cores and workload
DATABASE_MIN_CONNECTIONS=5      # Maintain some idle connections
DATABASE_MAX_LIFETIME=1800      # Connection lifetime in seconds
DATABASE_IDLE_TIMEOUT=600       # Idle connection timeout in seconds
```

### 3. Middleware Optimization

1. **Rate Limiting**: Adjust rate limits based on actual API usage patterns

2. **Compression**: The application uses Gzip and Brotli compression. Configure the compression level:

```rust
// In router.rs
CompressionLayer::new()
    .gzip(true)
    .gzip_level(6)  // Balance between speed and compression (1-9)
    .br(true)
    .br_level(3)    // Balance between speed and compression (1-11)
```

3. **Request Logging**: Consider disabling debug-level request logging in production

### 4. Template Caching

Ensure template caching is enabled in production:

```bash
TEMPLATE_CACHE_ENABLED=true
```

## Database Optimization

### 1. Index Optimization

Ensure the following indexes are present:

```sql
-- Users table
CREATE INDEX idx_username ON users(username);
```

### 2. Query Optimization

1. **Use Prepared Statements**: The application already uses sqlx's prepared statements, but verify all database access uses this pattern

2. **Limit Query Results**: Always limit the number of rows returned

```rust
// Example of proper pagination
let users = sqlx::query_as!(
    User,
    "SELECT * FROM users ORDER BY username LIMIT ? OFFSET ?",
    limit,
    offset
)
.fetch_all(&pool)
.await?;
```

3. **Select Only Required Columns**:

```rust
// Instead of SELECT *
let name = sqlx::query_scalar!(
    "SELECT display_name FROM users WHERE username = ?",
    username
)
.fetch_one(&pool)
.await?;
```

### 3. Connection Configuration

Configure MySQL for optimal performance:

```bash
# MySQL configuration (my.cnf)
innodb_buffer_pool_size = 1G           # Adjust based on available RAM
innodb_log_buffer_size = 16M           # Log buffer size
max_connections = 150                  # Maximum connections
query_cache_size = 64M                 # Query cache size
query_cache_type = 1                   # Enable query cache
```

## Front-End Performance

### 1. HTML Optimization

1. **Minimize HTML Size**: Keep HTML responses compact
2. **Defer JavaScript Execution**: Use the `defer` attribute for non-critical scripts
3. **Preconnect to Origins**: Add preconnect hints for external resources

```html
<!-- Example of optimized HTML header -->
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Page Title</title>
  <link rel="preconnect" href="https://api.example.com">
  <style>
    /* Critical CSS inline */
    body { margin: 0; font-family: system-ui, sans-serif; }
  </style>
  <script defer>
    // Critical initialization
    document.addEventListener('DOMContentLoaded', () => {
      // Initialize components
    });
  </script>
</head>
```

### 2. Asset Optimization

1. **Minify CSS and JavaScript**: Ensure all CSS and JavaScript is minified in production
2. **Optimize Images**: Use appropriate image formats and sizes
3. **Cache Headers**: Configure proper cache headers for static assets

```
# Nginx configuration
location /static/ {
    expires 30d;
    add_header Cache-Control "public, max-age=2592000, immutable";
}
```

## Caching Strategies

### 1. Response Caching

Configure cache headers for API responses:

```rust
// For immutable resources (e.g., by ID)
response.headers_mut().insert(
    header::CACHE_CONTROL,
    header::HeaderValue::from_static("public, max-age=3600"),
);

// For dynamic resources
response.headers_mut().insert(
    header::CACHE_CONTROL,
    header::HeaderValue::from_static("private, max-age=60"),
);
```

### 2. Database Result Caching

Use in-memory caching for frequently accessed, rarely changing data:

```rust
// Example with a simple LRU cache
use std::sync::Arc;
use lru::LruCache;
use std::sync::Mutex;

// Cache configuration
let user_cache = Arc::new(Mutex::new(LruCache::new(100))); // 100 items

// Cache usage
let username = "user123";
let user = {
    let mut cache = user_cache.lock().unwrap();
    if let Some(user) = cache.get(username) {
        user.clone()
    } else {
        let user = db.get_user(username).await?;
        cache.put(username.to_string(), user.clone());
        user
    }
};
```

### 3. Static Content Delivery

Use a CDN or nginx caching for static content:

```
# Nginx configuration with proxy_cache
proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=app_cache:10m max_size=1g inactive=60m;

server {
    location / {
        proxy_cache app_cache;
        proxy_cache_valid 200 302 10m;
        proxy_cache_valid 404 1m;
    }
}
```

## Performance Testing

### 1. Load Testing

Use `wrk` or `k6` for load testing:

```bash
# Basic load test with wrk
wrk -t12 -c400 -d30s http://localhost:3000/api/username/test_user

# Performance benchmark with just command
just benchmark
```

Interpret results:

- Latency: Average response time
- Requests/sec: Throughput
- Transfer/sec: Bandwidth usage

### 2. Profile CPU Usage

Use the flamegraph tool to identify CPU hotspots:

```bash
just profile
```

Common hotspots to look for:
- Expensive database queries
- Inefficient serialization/deserialization
- Template rendering bottlenecks
- Repeated calculations that could be cached

### 3. Memory Profiling

Monitor memory usage with:

```bash
just prod-up
docker stats
```

Look for:
- Steady increases in memory usage (potential leaks)
- Excessive memory usage during peak loads
- Frequent garbage collection

## Production Optimization

### 1. Container Configuration

Optimize Docker containers:

```yaml
# In compose.yml
services:
  app_prod:
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 1G
        reservations:
          cpus: '2'
          memory: 512M
```

### 2. Network Optimization

1. **Nginx Configuration**:

```
# Nginx optimization
worker_processes auto;
worker_connections 1024;
keepalive_timeout 65;
sendfile on;
tcp_nodelay on;
tcp_nopush on;
gzip on;
gzip_comp_level 6;
gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
```

2. **HTTP/2 Support**:

```
# Enable HTTP/2 in Nginx
listen 443 ssl http2;
```

### 3. OS Tuning

Optimize Linux kernel parameters:

```bash
# /etc/sysctl.conf optimizations
net.core.somaxconn = 4096
net.ipv4.tcp_max_syn_backlog = 4096
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_max_tw_buckets = 1440000
```

## Monitoring and Continuous Optimization

### 1. Set Up Monitoring Dashboard

Use Grafana and Prometheus to visualize:

- Request rates and response times
- Database query performance
- Memory and CPU usage
- Cache hit/miss ratios

### 2. Set Alerts for Performance Degradation

Configure alerts for:
- Response times exceeding thresholds
- Error rate increases
- Database connection exhaustion
- Memory leaks

### 3. Regular Performance Reviews

Schedule monthly performance reviews:
1. Review monitoring dashboards
2. Analyze slow queries
3. Check cache effectiveness
4. Update performance targets
5. Run load tests to validate improvements
