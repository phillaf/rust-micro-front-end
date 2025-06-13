# Requirements Document - Rust Micro Front-End Application

## Project Overview

This project demonstrates a minimum viable prototype for building micro web-applications that can be part of a larger-scale ecosystem. The application showcases how to meet specific performance, architectural, and development constraints while maintaining simplicity and composability.

## Product Specification

### Core Entity
- **User Entity** with:
  - `username`: Unique identifier (Twitter-style handle)
  - `display_name`: User's display name (single field)

### Web Components
1. **CMS Component**: User input interface for editing display name
   - JWT authentication required for editing
   - Data persistence to MySQL database
   - Users can only edit their own names

2. **Display Component**: Name rendering interface
   - Public read access (no authentication required)
   - Username passed as plain text parameter
   - Anyone can view any user's display name

### Authentication Model
- JWT-based authentication for write operations
- No JWT creation logic in the application
- Test scripts will generate authentication tokens
- Tokens passed via Authorization headers

## Technical Constraints

### Constraint 1: End-User Performance
- **Target**: Lighthouse score of 100
- **Server-Side Rendering (SSR)**: Maximum server-side rendering
- **Modern Browser Features**: Leverage HTTP/3, streaming, multiplexing, server caching, lazy loading
- **Optimization**: Minified and optimized pages

### Constraint 2: Tool Selection Philosophy
- **Efficiency First**: Low-level, efficient tools over bloated frameworks
- **Composability**: Independent, powerful, flexible tools
- **Minimal Dependencies**: Avoid large packages with excessive feature sets
- **Performance**: Top-tier server performance with resource-efficient tools

### Constraint 3: Self-Contained Environment
- **No CDNs**: All assets served locally
- **Docker-Based**: Complete setup runnable locally with Docker
- **Development Tools**: All development commands executable within Docker containers
- **Local Requirements**: Only `just`, `docker`, and `docker-compose` required locally

### Constraint 4: Micro Front-End Architecture
- **Web Components**: Two exposable micro front-ends
- **Composability**: Designed for assembly with other micro-apps on single webpage
- **Isolation**: Self-contained functionality

### Constraint 5: Simplicity Focus
- **No CDN Dependencies**
- **Inline JavaScript**: All JS held inline (no module loading)
- **No ORMs**: Direct database interaction

### Constraint 6: Development Toolkit
- **Development Mode**: Unminified page rendering capability
- **Dependency Injection**: Mock support for testing
- **Granular Environment Variables**: See `.env.example` for complete configuration reference

## Technology Stack

### Core Infrastructure
- **Development Tool**: `just` - Command runner for Docker container execution
- **Reverse Proxy**: `nginx` - HTTP server, logging, caching, SSL certificate management
- **Backend Language**: Rust - High-performance systems programming language
- **Database**: MySQL 8.0+ - Relational database with full ACID compliance
- **Containerization**: Docker + Docker Compose - Development and deployment isolation

### Rust Dependencies (Final)
- **Web Framework**: `axum` (preferred over warp) - Lightweight, async HTTP server
- **JWT Library**: `jsonwebtoken` - RS256/ES256 signature validation
- **Database Layer**: `sqlx` - Async, compile-time checked SQL queries (no ORM)
- **Templating Engine**: `minijinja` (preferred over handlebars) - Lightweight, Jinja2-compatible runtime templating
- **Serialization**: `serde` + `serde_json` - JSON serialization/deserialization for APIs
- **Error Handling**: `anyhow` - Flexible error handling with context
- **Input Validation**: `validator` - Struct-based validation with custom rules
- **Correlation IDs**: `uuid` - UUID generation for request tracing
- **Metrics Collection**: `prometheus` - Standard metrics and monitoring
- **Async Runtime**: `tokio` - Asynchronous runtime for Rust
- **Testing Framework**: `tokio-test` + `sqlx-test` - Async testing utilities

### Technology Selection Rationale
- **Axum over Warp**: Better ecosystem integration and ergonomics
- **Minijinja over Handlebars**: Lighter weight, better Rust integration, built-in XSS protection
- **sqlx over Diesel**: Compile-time query verification without ORM complexity
- **MySQL over PostgreSQL**: Proven performance and operational simplicity
- **Prometheus over Custom Metrics**: Industry standard monitoring integration
- **Anyhow over thiserror**: Simpler error handling for application-level errors
- **Environment Variables over Config Files**: Docker Compose handles .env injection directly

### Infrastructure
- **Database**: MySQL 8.0+
- **Containerization**: Docker + Docker Compose
- **Reverse Proxy**: nginx

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     nginx       │    │   Rust App      │    │     MySQL       │
│  (reverse proxy)│◄──►│   (axum/warp)   │◄──►│   (database)    │
│   - caching     │    │   - JWT auth    │    │   - user data   │
│   - SSL         │    │   - templating  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Templating Strategy

### Runtime Templating Engine Required
- **Rationale**: Application will be built once and deployed to server, but must render dynamic content at runtime based on database queries
- **Build-time vs Runtime**: Build-time templating engines (askama, tera) are inadequate as they cannot incorporate database data fetched at request time  
- **Proposed Engines**: 
  - `handlebars` - Mature, feature-rich, well-documented
  - `minijinja` - Lightweight, Jinja2-compatible, minimal dependencies
- **Data Flow**: Request → Database Query → Template Rendering → HTML Response (single request cycle)

### Server-Side Rendering Philosophy
- **Zero API Calls for Initial Load**: Initial page load contains all necessary data embedded in HTML
- **Subsequent API Calls Allowed**: Form submissions and updates can use API calls to avoid full page re-renders
- **Performance**: Eliminates loading states on initial load, reduces Time to Interactive
- **SEO Friendly**: Content immediately available to crawlers
- **Lighthouse Optimization**: Supports achieving 100/100 score by avoiding client-side data fetching on initial load

## API Endpoints

### Server-Side Rendered Pages (Initial Load)
- `GET /edit` - Render CMS form with current data embedded (requires JWT, username from token)
- `GET /display/username/{username}` - Render display name with current data embedded (public)

### API Endpoints (Subsequent Updates)
- `POST /api/username` - Update display name via API (requires JWT, username from token)
- `GET /api/username/{username}` - Fetch display name data as JSON (public)

### Utility Endpoints
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics

**Architecture Note**: Initial page loads are server-side rendered with data embedded. Subsequent updates can use API endpoints to avoid full page re-renders, providing a better user experience for form submissions and dynamic updates.

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    username VARCHAR(50) PRIMARY KEY,
    display_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### Schema Design Decisions
- **Primary Key**: `username` as VARCHAR(50) to support Twitter-style handles
- **Display Name**: VARCHAR(100) for user-friendly display names
- **Audit Fields**: `created_at` and `updated_at` for change tracking
- **Constraints**: `display_name` is NOT NULL to ensure data integrity
- **MySQL Features**: Uses `ON UPDATE CURRENT_TIMESTAMP` for automatic timestamp updates

### Data Validation Rules
- **Username Format**: Alphanumeric characters, underscores, and hyphens only
- **Username Length**: 3-50 characters (enforced at application level)
- **Display Name Length**: 1-100 characters (enforced at application level)
- **Character Encoding**: UTF-8 support for international characters in display names

## Development Workflow

### Commands (via `just`)
- `just build` - Build application in Docker
- `just dev` - Start development environment
- `just test` - Run test suite
- `just migrate` - Run database migrations
- `just format` - Format code
- `just lint` - Run linting

### Environment Modes
- **Granular Configuration**: Each feature controlled by independent environment variables
- **No Environment Presets**: Avoid umbrella variables like `ENVIRONMENT=prod`
- **Runtime Flexibility**: 
  - Development: `ENABLE_MINIFICATION=false`, `ENABLE_DEBUG_LOGGING=true`, `DATABASE_ADAPTER=mock`
  - Production: `ENABLE_MINIFICATION=true`, `ENABLE_DEBUG_LOGGING=false`, `DATABASE_ADAPTER=mysql`
  - Custom configurations possible by mixing any combination of feature flags

## Performance Targets

- **Lighthouse Score**: 100/100
- **First Contentful Paint**: < 1.2s
- **Largest Contentful Paint**: < 2.5s
- **Time to Interactive**: < 3.8s
- **Cumulative Layout Shift**: < 0.1

## Security Considerations

### JWT Implementation
- **Public key validation** only (no token generation logic in application)
- **Signature algorithms**: RS256/ES256 (asymmetric cryptography)
- **Username claim validation** - ensure users can only edit their own data
- **Token expiration** handling and validation
- **Invalid token** graceful handling and error responses

### Input Validation and Sanitization
- **Username format** validation (Twitter-style handles: alphanumeric, underscores, hyphens)
- **Username length** limits (3-50 characters)
- **Display name** sanitization and length limits (1-100 characters)
- **HTML entity encoding** for all user-generated content
- **SQL injection prevention** through parameterized queries exclusively
- **XSS protection** through template escaping (automatic in handlebars/minijinja)

### Network Security
- **HTTPS enforcement** in production environments
- **CORS configuration** for micro front-end integration
- **Security headers** (CSP, X-Frame-Options, X-Content-Type-Options)
- **Rate limiting** on authentication endpoints
- **Request size limits** to prevent DoS attacks

### Database Security
- **Least privilege** database user with minimal required permissions
- **Connection encryption** (TLS) for database connections
- **No sensitive data logging** in database queries
- **Prepared statements** only (no dynamic SQL construction)

### Application Security
- **Secure defaults** for all configuration options
- **Environment variable validation** on startup
- **Graceful error handling** without information leakage
- **Audit logging** for authentication attempts and data modifications

## Monitoring and Observability

### Metrics Collection (Prometheus)
- **HTTP Request Metrics**:
  - Request count by endpoint and status code
  - Request duration histograms
  - Active request gauges
- **Authentication Metrics**:
  - JWT validation success/failure rates
  - Authentication attempt counts
  - Token expiration events
- **Database Metrics**:
  - Query execution time histograms
  - Connection pool utilization
  - Transaction success/failure rates
- **Application Metrics**:
  - Template rendering duration
  - Cache hit/miss ratios
  - Error rates by component

### Structured Logging
- **Request Tracing**:
  - Correlation IDs for request tracking
  - User context (username) in logs
  - Request/response payload sizes
- **Error Context**:
  - Stack traces with correlation IDs
  - Input data context (sanitized)
  - System state at error time
- **Performance Markers**:
  - Database query execution times
  - Template rendering times
  - Cache operation latencies
- **Security Events**:
  - Authentication failures
  - Authorization denials
  - Input validation failures

### Health Monitoring
- **Application Health**:
  - `/health` endpoint with comprehensive checks
  - Database connectivity verification
  - JWT public key validation
  - Template engine initialization status
- **System Health**:
  - Memory usage monitoring
  - CPU utilization tracking
  - Disk space monitoring
  - Network connectivity checks
- **Performance Health**:
  - Response time SLA monitoring
  - Error rate thresholds
  - Resource utilization limits

### Alerting Strategy (Data Contract Only)
This application **does not implement alerting logic internally**. Instead, it exposes standardized metrics and logs for external monitoring tools (e.g., Grafana) to consume and alert on.

**Alerting Thresholds for External Tools**:
- **Critical Alerts**:
  - Application unavailability (health check failures)
  - Database connection failures (connection pool exhaustion)
  - Authentication system failures (JWT validation errors >5%)
- **Warning Alerts**:
  - High error rates (>1% of requests returning 5xx status)
  - Slow response times (>2s average response time)
  - Resource utilization (>80% memory or CPU usage)
- **Information Alerts**:
  - New user registrations (first-time username creation)
  - Unusual traffic patterns (request rate >10x baseline)
  - Configuration changes (environment variable modifications)

**External Tool Integration**:
- **Prometheus metrics** exposed at `/metrics` endpoint
- **Structured logging** with correlation IDs for alert context
- **Standardized metric names** for consistent alerting across micro-apps
- **Health check endpoint** at `/health` for availability monitoring

## Testing Strategy

- **Unit Tests**: Business logic and utilities
- **Integration Tests**: API endpoints with mock database
- **End-to-End Tests**: Full user workflows
- **Performance Tests**: Load testing and profiling

## Deployment Architecture

```yaml
# docker-compose.yml structure
services:
  nginx:
    - Port 80/443 exposure
    - SSL termination
    - Static asset serving
    - Request proxying
  
  app:
    - Rust application
    - Internal port 3000
    - Granular environment configuration:
      - DATABASE_ADAPTER=mysql
      - ENABLE_MINIFICATION=true
      - ENABLE_DEBUG_LOGGING=false
      - ENABLE_METRICS=true
      - ENABLE_CACHING=true
      - LOG_LEVEL=info
  
  mysql:
    - Database service
    - Volume persistence
    - Database connection variables
```

### Configuration Philosophy
- **No Environment Presets**: Each feature independently configurable
- **Explicit Control**: Every behavior controlled by specific environment variable
- **Runtime Flexibility**: Same binary can behave differently based on configuration
- **Debugging Capability**: Individual features can be toggled for troubleshooting

## Success Criteria

1. **Functional**: Both web components work independently and together
2. **Performance**: Lighthouse score of 100
3. **Containerized**: Complete Docker-based development workflow
4. **Composable**: Web components integrate with other micro front-ends
5. **Maintainable**: Clear separation of concerns and testable architecture

## Future Considerations

- Multi-tenancy support
- Horizontal scaling patterns
- Advanced caching strategies
- Real-time updates (WebSocket/SSE)
- Observability and monitoring enhancements
