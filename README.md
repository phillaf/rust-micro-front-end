# Rust Micro Front-End Application

A high-performance, containerized micro web-application demonstrating modern web development constraints with Rust, Docker, and micro front-end architecture.

## Project Overview

This project showcases a minimum viable prototype for micro web-applications that can be composed into larger ecosystems. It features two web components (CMS and Display) for managing user display names with JWT authentication, server-side rendering, and optimal performance.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     nginx       │    │   Rust App      │    │     MySQL       │
│  (reverse proxy)│◄──►│   (axum/warp)   │◄──►│   (database)    │
│   - caching     │    │   - JWT auth    │    │   - user data   │
│   - SSL         │    │   - templating  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

For detailed architecture decisions, technology stack rationale, and database schema, see [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md).

## Getting Started

This project uses a fully containerized development environment. You don't need to install Rust, MySQL, or any development tools locally.

### Prerequisites

- `just` - Command runner
- `docker` - Container runtime
- `docker-compose` - Container orchestration

### Setup

1. **Copy environment configuration**:
   ```bash
   cp .env.example .env
   ```

2. **Customize environment** (optional):
   Edit `.env` for your specific needs. The defaults are configured for development.

3. **Start development environment**:
   ```bash
   just dev
   ```

### Quick Start

```bash
# Setup environment
cp .env.example .env

# Start development environment
just dev

# View the application
open http://localhost

# Monitor logs
just logs
```

### Development Commands

All development commands are executed through the `justfile`. For the complete list of available commands, see the [justfile](justfile) in the root directory.

Key commands include:
- `just dev` - Start development environment
- `just test` - Run test suite
- `just build` - Build application
- `just format` - Format code
- `just lint` - Run linting

## Development Workflow

### Making Changes

1. **Edit source code** - Hot reload handles rebuilds automatically
2. **Run tests** - `just test` validates your changes
3. **Check formatting** - `just format` before committing
4. **Verify build** - `just build` ensures clean compilation

### Database Changes

1. **Create migration** in `migrations/` directory
2. **Apply changes** with `just migrate`
3. **Update models** in `src/database/` if needed
4. **Test compatibility** with `just test`

### Environment Configuration

The application uses granular environment variables for precise control over features and behavior.

Copy `.env.example` to `.env` and customize as needed:
```bash
cp .env.example .env
```

The `.env.example` file contains:
- **Smart development defaults** - Ready to use for local development
- **Detailed configuration comments** - Explains each environment variable
- **Example configurations** - Development, production, and testing modes
- **Validation requirements** - Acceptable values for each variable

All environment variables are validated at application startup to ensure correct configuration.

### Development Modes

The application supports flexible configuration through granular environment variables. See `.env.example` for complete configuration examples including development, production, and testing modes.

### Performance Architecture

- **Zero client-side API calls for initial page load** - All data embedded in initial HTML
- **Subsequent API calls for updates** - Avoid full page re-renders for form submissions
- **Server-side rendering** - Dynamic content rendered at request time
- **Inline JavaScript** - No module loading complexity
- **Aggressive caching** - nginx and application-level caching
- **Modern HTTP features** - HTTP/3, streaming, multiplexing support

## Technology Stack

For detailed technology stack information, selection rationale, and dependency specifications, see [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md).

## Testing

The project includes comprehensive testing with containerized test environments:

- **Unit Tests** - Business logic and utilities
- **Integration Tests** - API endpoints with mock database
- **End-to-End Tests** - Complete user workflows
- **Performance Tests** - Load testing and benchmarking
- **Security Tests** - JWT validation and input sanitization

All tests run in isolated Docker containers with mock databases for fast iteration.

## License

This project is dual-licensed:
- **Non-commercial use**: Free under custom license terms
- **Commercial use**: Requires separate commercial license

See [LICENSE](LICENSE) for complete terms.

## Contributing

This project uses a **fully containerized development environment**. You don't need to install Rust, MySQL, cargo, rustfmt, clippy, or any other development tools on your local machine. Everything runs inside Docker containers.

### Contribution Workflow
1. **Fork** the repository
2. **Create** a feature branch
3. **Use** `just dev` for development with hot reload
4. **Test** with `just test` to verify changes
5. **Format** with `just format` before committing
6. **Submit** a pull request

### Code Quality Standards
- **Formatting**: Use `just format` before committing
- **Linting**: Use `just lint` to check code quality
- **Testing**: Use `just test` to verify all functionality
- **Security**: Run `just audit` for dependency security checks

### Development Tools
- **Mock Database**: Use `DATABASE_ADAPTER=mock` for fast iteration
- **Granular Logging**: Adjust `LOG_LEVEL` for debugging needs
- **Feature Flags**: Use individual `ENABLE_*` variables for testing

## Monitoring & Observability

The application includes comprehensive monitoring and observability features:

### Prometheus Metrics

Access Prometheus metrics at `/metrics` endpoint. Available metrics include:

- **HTTP Metrics**:
  - `http_requests_total` - Request count by method, path, and status
  - `http_requests_duration_seconds` - Request duration histograms
  - `http_requests_in_flight` - Currently active requests

- **Authentication Metrics**:
  - `auth_success_total` - Successful authentication attempts
  - `auth_failure_total` - Failed authentication attempts by reason

- **Database Metrics**:
  - `database_queries_total` - Query count by operation and status
  - `database_query_duration_seconds` - Query execution time histograms

- **Application Metrics**:
  - `template_render_duration_seconds` - Template rendering performance
  - `cache_hit_total` and `cache_miss_total` - Cache effectiveness

### Health Monitoring

The `/health` endpoint provides comprehensive health checks:

- **Database connectivity**
- **Template engine status**
- **JWT public key validation**
- **Uptime tracking**
- **Environment verification**

```bash
# Sample health check request
curl http://localhost/health | jq
```

### Grafana Integration

The Prometheus metrics can be visualized using Grafana:

1. Configure Grafana to use the `/metrics` endpoint as a Prometheus data source
2. Import the dashboards from `monitoring/dashboards/` (coming soon)
3. Set up alerts based on performance thresholds

