# Troubleshooting Guide

This guide provides solutions to common issues you might encounter when working with the Rust Micro Front-End Application.

## Table of Contents

1. [Development Environment Issues](#development-environment-issues)
2. [Docker and Container Issues](#docker-and-container-issues)
3. [Authentication Problems](#authentication-problems)
4. [Database Connectivity](#database-connectivity)
5. [Performance Issues](#performance-issues)
6. [API Errors](#api-errors)
7. [Production Deployment Issues](#production-deployment-issues)

## Development Environment Issues

### Command 'just' not found

**Problem**: The `just` command-line tool is not installed.

**Solution**: Install the Just command runner:

```bash
# On Debian/Ubuntu
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | sudo bash -s -- --to /usr/local/bin

# On macOS
brew install just

# On Windows with Chocolatey
choco install just
```

### Docker permission issues

**Problem**: Permission denied when running Docker commands.

**Solution**: Add your user to the Docker group:

```bash
sudo usermod -aG docker $USER
newgrp docker
```

### Workspace volume not syncing changes

**Problem**: Changes made to files aren't reflected in the container.

**Solution**: 
1. Check Docker file sharing permissions
2. Restart Docker service
3. Rebuild the container:
   ```bash
   just clean
   just build
   ```

## Docker and Container Issues

### Container exits immediately

**Problem**: The Docker container exits immediately after starting.

**Solution**: Check the logs for errors:

```bash
docker compose logs app
```

Common causes:
- Environment variables not set properly
- Port conflicts
- Permission issues

### Out of disk space errors

**Problem**: Docker operations fail due to lack of disk space.

**Solution**: Clean up unused Docker resources:

```bash
docker system prune -a
```

### Container performance issues

**Problem**: Container performance is unexpectedly slow.

**Solution**:
1. Check Docker resource allocation (CPU/memory)
2. On macOS or Windows, check Docker Desktop settings
3. Optimize the Dockerfile to use multi-stage builds
4. Review volume mounts that might cause I/O bottlenecks

## Authentication Problems

### JWT verification failures

**Problem**: JWT token validation fails with errors.

**Solution**:

1. Check that the public key path is correctly set in environment variables:
   ```
   JWT_PUBLIC_KEY_PATH=/app/scripts/jwt_public_key.pem
   ```

2. Verify that the key files have the correct permissions:
   ```bash
   chmod 400 scripts/jwt_private_key.pem
   chmod 444 scripts/jwt_public_key.pem
   ```

3. Ensure the token is properly formatted and not expired:
   ```bash
   just bash
   bash /app/scripts/jwt_test_helper.sh decode <your-token>
   ```

4. For testing, you can generate a new token:
   ```bash
   just bash
   bash /app/scripts/jwt_test_helper.sh generate username
   ```

### Authentication headers not sent properly

**Problem**: API returns 401 Unauthorized even with a valid token.

**Solution**:
1. Ensure the Authorization header is correctly formatted:
   ```
   Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
   ```

2. Check that the token is not URL-encoded or otherwise malformed

3. For debugging, use the token debug endpoint:
   ```
   GET /debug/set-token/username
   ```

## Database Connectivity

### Failed to connect to MySQL

**Problem**: Application fails to connect to the MySQL database.

**Solution**:
1. Check database environment variables:
   ```
   DATABASE_URL=mysql://user:password@mysql:3306/dbname
   ```

2. Verify MySQL container is running:
   ```bash
   docker compose ps mysql
   ```

3. Try connecting manually to diagnose:
   ```bash
   just bash
   mysql -h mysql -u user -p dbname
   ```

4. Run migrations to ensure database schema is up to date:
   ```bash
   just migrate
   ```

### Migration failures

**Problem**: Database migrations fail to apply.

**Solution**:
1. Check migration logs:
   ```bash
   just migrate-debug
   ```

2. Ensure migrations are numbered correctly
3. Check for SQL syntax errors in migration files
4. For testing, reset the database:
   ```bash
   just reset-db
   ```

## Performance Issues

### Slow response times

**Problem**: API endpoints respond slower than expected.

**Solution**:
1. Check the metrics endpoint for performance insights:
   ```
   GET /metrics
   ```

2. Look for slow database queries in logs
3. Check for missing database indexes
4. Review template rendering performance
5. Enable compression if not already enabled

### High memory usage

**Problem**: Application uses more memory than expected.

**Solution**:
1. Check for memory leaks in custom code
2. Optimize database connection pool settings
3. Review template caching configuration
4. Monitor with Docker stats:
   ```bash
   docker stats
   ```

## API Errors

### 400 Bad Request errors

**Problem**: API endpoints return 400 Bad Request.

**Solution**:
1. Check request payload format
2. Verify input validation rules:
   - Username: 3-50 characters, alphanumeric plus underscore and hyphen
   - Display name: 1-100 characters, sanitized input
3. Ensure Content-Type header is set correctly:
   ```
   Content-Type: application/json
   ```

### 429 Too Many Requests errors

**Problem**: API endpoints return 429 Too Many Requests.

**Solution**:
1. Rate limiting is enforced on authenticated endpoints
2. Implement proper backoff strategy in clients
3. For testing, you can modify rate limits in the code (not recommended for production)

## Production Deployment Issues

### SSL/TLS certificate errors

**Problem**: HTTPS connections fail with certificate errors.

**Solution**:
1. Check certificate paths in nginx configuration
2. Verify certificates are valid and not expired:
   ```bash
   openssl x509 -text -noout -in /path/to/cert.pem
   ```
3. For testing, regenerate self-signed certificates:
   ```bash
   just generate-ssl
   ```

### Static file serving issues

**Problem**: Static files (robots.txt, sitemap.xml, etc.) not served properly.

**Solution**:
1. Verify nginx configuration for static file locations
2. Check file permissions on static files
3. Ensure files exist in the correct location

### Container health check failures

**Problem**: Docker health checks fail in production.

**Solution**:
1. Check logs for specific health check failures:
   ```bash
   just prod-logs
   ```
2. Verify the health endpoint is responding correctly
3. Increase health check timeout if the application has a slow startup

### "Address already in use" errors

**Problem**: Container fails to start due to port conflicts.

**Solution**:
1. Check for other services using the required ports:
   ```bash
   sudo netstat -tulpn | grep 80
   sudo netstat -tulpn | grep 443
   ```
2. Stop conflicting services or change ports in compose.yml
3. Ensure previous instances are properly shut down:
   ```bash
   just prod-down
   ```
