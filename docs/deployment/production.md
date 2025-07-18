# Production Deployment Guide

This guide explains how to deploy the Rust Micro Front-End Application to a production environment.

## Prerequisites

- Docker and Docker Compose installed on the host machine
- Access to the Git repository
- A domain name (for SSL certificates in production)
- Proper environment variables configured (see Environment Configuration section)

## Deployment Steps

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/rust-micro-front-end.git
cd rust-micro-front-end
```

### 2. Configure environment variables

Create a `.env` file with production settings:

```bash
# Copy the example file and edit it
cp .env.example .env
```

Edit the following key variables in `.env`:

- `DATABASE_ADAPTER=mysql`
- `DATABASE_HOST=mysql`
- `DATABASE_NAME=micro_frontend`
- `DATABASE_USERNAME=[secure username]`
- `DATABASE_PASSWORD=[secure password]`
- `JWT_PUBLIC_KEY=[your JWT public key]`
- `ENABLE_DEBUG_LOGGING=false`
- `MINIFY_ENABLED=true`
- `ENABLE_CACHING=true`

For MySQL container:
- `MYSQL_ROOT_PASSWORD=[secure root password]`
- `MYSQL_PASSWORD=[secure user password]`

### 3. Configure SSL certificates

For production, you should use real SSL certificates from a trusted Certificate Authority.

1. Obtain SSL certificates for your domain
2. Place the certificate files in the `nginx/certs` directory:
   - `server.crt` - SSL certificate
   - `server.key` - SSL private key

For testing/development, you can generate self-signed certificates:

```bash
just generate-ssl
```

### 4. Build production Docker image

```bash
just build-prod
```

### 5. Start the production environment

```bash
just prod-up
```

### 6. Run database migrations

```bash
just prod-migrate
```

### 7. Verify deployment

Check that all services are running correctly:

```bash
docker compose --profile prod ps
```

Access the application at `https://your-domain.com`

## Monitoring and Maintenance

### Viewing logs

```bash
just prod-logs
```

### Stopping the application

```bash
just prod-down
```

### Health check endpoint

The application exposes a health check endpoint at `/health`. You can use this to monitor the application's health:

```bash
curl https://your-domain.com/health
```

### Prometheus metrics

Metrics are available at the `/metrics` endpoint (restricted by IP in the nginx configuration):

```bash
curl https://your-domain.com/metrics
```

## Security Considerations

- The production Docker image runs as a non-root user
- The container has minimal permissions and a read-only filesystem
- Security headers are configured in nginx
- Rate limiting is enabled to prevent abuse
- JWT authentication is required for sensitive operations

## Backup and Recovery

### Database Backup

MySQL data is persisted in a Docker volume. To create a backup:

```bash
docker compose exec mysql mysqldump -u root -p micro_frontend > backup.sql
```

### Restore from Backup

```bash
cat backup.sql | docker compose exec -T mysql mysql -u root -p micro_frontend
```

## Troubleshooting

### Connection Issues

If you can't connect to the application:

1. Check if all containers are running:
   ```bash
   docker compose --profile prod ps
   ```

2. Check the logs for errors:
   ```bash
   just prod-logs
   ```

3. Verify the health of the application:
   ```bash
   curl https://your-domain.com/health
   ```

### Database Connection Errors

If the application can't connect to the database:

1. Check MySQL container logs:
   ```bash
   docker compose logs mysql
   ```

2. Verify environment variables in the `.env` file

### SSL Certificate Issues

If you encounter SSL certificate warnings:

1. Ensure you've placed valid certificates in `nginx/certs`
2. Check nginx logs for certificate-related errors:
   ```bash
   docker compose logs nginx
   ```

## Performance Tuning

- Adjust `DATABASE_MAX_CONNECTIONS` based on expected load
- Configure cache TTL settings as needed
- Monitor resource usage and adjust container resource limits
