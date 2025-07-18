# Container Security Hardening Guide

This document outlines best practices for securing Docker containers for the Rust Micro Front-End Application.

## Container Security Principles

1. **Minimal Attack Surface**: Reduce the components and features available in the container
2. **Least Privilege**: Run containers with the minimum required permissions
3. **Immutability**: Treat containers as immutable and avoid runtime changes
4. **Defense in Depth**: Apply multiple layers of security controls
5. **Regular Updates**: Keep base images and dependencies updated

## Docker Image Security

### 1. Use Minimal Base Images

The application's production Dockerfile uses:
- `debian:bookworm-slim` as the base image
- Multi-stage builds to minimize the final image size

Benefits:
- Reduced attack surface
- Smaller image size
- Faster deployment

### 2. Run as Non-Root User

```dockerfile
# Create a non-root user
RUN groupadd -r app && useradd -r -g app app

# Switch to non-root user
USER app
```

Benefits:
- Prevents container breakout exploits
- Limits damage from compromised applications

### 3. Use Read-Only Filesystem

```dockerfile
# In docker-compose.yml
read_only: true
```

Benefits:
- Prevents malicious file writes
- Makes the container more immutable

### 4. Drop Unnecessary Capabilities

```dockerfile
# In docker-compose.yml
cap_drop:
  - ALL
```

Benefits:
- Prevents privilege escalation
- Restricts container operations to necessary functions

### 5. Use Security Options

```dockerfile
# In docker-compose.yml
security_opt:
  - no-new-privileges:true
```

Benefits:
- Prevents privilege escalation through setuid binaries
- Limits potential exploit vectors

## Runtime Security

### 1. Resource Limits

```dockerfile
# In docker-compose.yml
deploy:
  resources:
    limits:
      cpus: '1'
      memory: 256M
```

Benefits:
- Prevents resource exhaustion attacks
- Ensures container isolation

### 2. Health Checks

```dockerfile
# In docker-compose.yml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost/health"]
  interval: 30s
  timeout: 10s
  retries: 3
```

Benefits:
- Ensures application availability
- Automatically restarts unhealthy containers

### 3. Security Scanning

Implement regular scanning:
- Vulnerability scanning using tools like Trivy or Clair
- Configuration scanning using Docker Bench for Security
- Runtime behavior monitoring

Example command:
```bash
# Run Trivy scanner on the production image
trivy image rust-micro-front-end:production
```

## Network Security

### 1. Container Network Isolation

```dockerfile
# In docker-compose.yml
networks:
  frontend:
    # External-facing network
  backend:
    # Internal network for app-db communication
    internal: true
```

Benefits:
- Network segmentation
- Defense in depth

### 2. Restrict Exposed Ports

Only expose necessary ports:

```dockerfile
# In docker-compose.yml
ports:
  - "443:443"  # Only expose HTTPS port
```

### 3. Use TLS for Communication

- HTTPS for client-server communication
- TLS for database connections

## Secrets Management

### 1. Use Docker Secrets

```dockerfile
# In docker-compose.yml
secrets:
  - db_password
  - jwt_public_key
```

Benefits:
- Secure handling of sensitive data
- Secrets not exposed in environment variables

### 2. Avoid Hardcoding Secrets

- Never hardcode secrets in Dockerfiles
- Use runtime injection of secrets

## Monitoring and Logging

### 1. Container Logging

```dockerfile
# In docker-compose.yml
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```

Benefits:
- Prevents log storage exhaustion
- Preserves important log data

### 2. Security Monitoring

- Enable auditing
- Monitor container behavior for anomalies

## Security Checklist

- [ ] Container runs as non-root user
- [ ] Read-only filesystem enabled
- [ ] Unnecessary capabilities dropped
- [ ] Resource limits implemented
- [ ] Health checks configured
- [ ] Security scanning automated
- [ ] Network isolation configured
- [ ] Secrets managed securely
- [ ] Container logging configured
- [ ] Security monitoring in place
- [ ] Base image regularly updated
- [ ] Dependencies regularly updated
- [ ] No sensitive data in image layers

## Security Testing

Regular security testing should include:
1. Container breakout attempts
2. Privilege escalation testing
3. Network isolation verification
4. Resource limit testing

## References

- [Docker Security Documentation](https://docs.docker.com/engine/security/)
- [NIST Container Security Guide](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-190.pdf)
- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker/)
