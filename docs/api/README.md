# API Documentation

This document outlines all API endpoints provided by the Rust Micro Front-End Application.

## Authentication

Most write operations require JWT authentication. To authenticate requests:

1. Include a valid JWT token in the `Authorization` header:

   ```text
   Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
   ```

2. The JWT must:
   - Be signed with RS256/ES256 algorithm
   - Contain a username claim that matches the resource being accessed
   - Not be expired
   - Have the correct audience and issuer claims

## Rate Limiting

Write operations and authenticated endpoints have rate limiting applied. Exceeding the rate limit will result in HTTP 429 (Too Many Requests) responses.

## API Endpoints

### User Display Name API

#### GET /api/username/{username}

Retrieves the display name for a specified username.

**Parameters:**

- `username` (path): The username to look up (3-50 characters, alphanumeric, underscores, hyphens)

**Authentication:**

- None (public endpoint)

**Response:**

```json
{
  "username": "john_doe",
  "display_name": "John Doe"
}
```

**Status Codes:**

- 200: Success
- 400: Invalid username format
- 404: Username not found
- 500: Server error

**Example:**

```bash
curl -X GET https://example.com/api/username/john_doe
```

#### POST /api/username

Updates the display name for the authenticated user.

**Parameters:**

- None (uses username from JWT token)

**Request Body:**

```json
{
  "display_name": "New Display Name"
}
```

**Authentication:**

- Required (JWT token with username claim)

**Response:**

```json
{
  "username": "john_doe",
  "display_name": "New Display Name"
}
```

**Status Codes:**

- 200: Success
- 400: Invalid request format or display name validation failed
- 401: Unauthorized (missing or invalid JWT)
- 403: Forbidden (JWT username claim doesn't match resource)
- 429: Too Many Requests (rate limit exceeded)
- 500: Server error

**Example:**

```bash
curl -X POST https://example.com/api/username \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"display_name": "New Display Name"}'
```

### Web Components

#### GET /display/username/{username}

Returns an HTML component that displays the username and display name. This is a server-side rendered component for micro front-end integration.

**Parameters:**

- `username` (path): The username to display (3-50 characters, alphanumeric, underscores, hyphens)

**Authentication:**

- None (public endpoint)

**Response:**

- HTML with embedded user data
- Content-Type: text/html

**Status Codes:**

- 200: Success (even if user not found, will show error in component)
- 400: Invalid username format
- 500: Server error

**Example:**

```bash
curl -X GET https://example.com/display/username/john_doe
```

#### GET /edit

Returns an HTML component for editing the display name of the authenticated user. This is a server-side rendered CMS component for micro front-end integration.

**Parameters:**

- None (uses username from JWT token)

**Authentication:**

- Required (JWT token with username claim)

**Response:**

- HTML with embedded form and user data
- Content-Type: text/html

**Status Codes:**

- 200: Success
- 401: Unauthorized (missing or invalid JWT)
- 429: Too Many Requests (rate limit exceeded)
- 500: Server error

**Example:**

```bash
curl -X GET https://example.com/edit \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### Static Resources

#### GET /manifest.json

Returns the web app manifest for PWA support.

**Authentication:**

- None (public endpoint)

**Response:**

- JSON manifest with application metadata
- Content-Type: application/json
- Cache-Control: public, max-age=3600

**Example:**

```bash
curl -X GET https://example.com/manifest.json
```

#### GET /robots.txt

Returns the robots.txt file for SEO.

**Authentication:**

- None (public endpoint)

**Response:**

- Plain text robots.txt content
- Content-Type: text/plain
- Cache-Control: public, max-age=86400

**Example:**

```bash
curl -X GET https://example.com/robots.txt
```

#### GET /sitemap.xml

Returns the sitemap XML for SEO.

**Authentication:**

- None (public endpoint)

**Response:**

- XML sitemap
- Content-Type: application/xml
- Cache-Control: public, max-age=86400

**Example:**

```bash
curl -X GET https://example.com/sitemap.xml
```

### System Endpoints

#### GET /health

Returns detailed health check information about the application and its dependencies.

**Parameters (optional):**

- `request_id` (query): Custom request ID for tracing

**Authentication:**

- None (public endpoint)

**Response:**

```json
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2025-07-17T12:00:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "uptime_seconds": 3600,
  "checks": {
    "database": {
      "status": "ok",
      "message": "Connected to MySQL",
      "timestamp": "2025-07-17T12:00:00Z"
    },
    "jwt_key": {
      "status": "ok",
      "message": "JWT public key is valid",
      "timestamp": "2025-07-17T12:00:00Z"
    },
    "template_engine": {
      "status": "ok",
      "message": "Template engine is operational",
      "timestamp": "2025-07-17T12:00:00Z"
    }
  }
}
```

**Status Codes:**

- 200: All systems operational
- 503: One or more systems degraded or unavailable
- 500: Server error

**Example:**

```bash
curl -X GET "https://example.com/health?request_id=my-custom-id"
```

#### GET /metrics

Returns Prometheus metrics for monitoring.

**Authentication:**

- None (in current implementation, but should be protected in production)

**Response:**

- Prometheus text format metrics
- Content-Type: text/plain

**Example:**

```bash
curl -X GET https://example.com/metrics
```

### Debug Endpoints

#### GET /debug/set-token/{username}

Debug utility to set a JWT token cookie in the browser. **This endpoint should be disabled in production.**

**Parameters:**

- `username` (path): The username to set in the token
- `token` (query, optional): Provide a specific token instead of generating one

**Authentication:**

- None (debug endpoint)

**Response:**

- HTML page with token information
- Sets authentication cookie

**Example:**

```bash
curl -X GET https://example.com/debug/set-token/john_doe
```

Health check endpoint for monitoring application status.

**Parameters:**
- None

**Authentication:**
- None (public endpoint)

**Response:**
```json
{
  "status": "ok",
  "version": "1.0.0",
  "uptime": "12h 34m 56s",
  "components": {
    "database": {
      "status": "ok",
      "message": "Connected to MySQL"
    },
    "template_engine": {
      "status": "ok"
    },
    "jwt_key": {
      "status": "ok"
    }
  }
}
```

**Status Codes:**
- 200: All components healthy
- 503: One or more components unhealthy

**Example:**
```bash
curl -X GET https://example.com/health
```

#### GET /metrics

Prometheus metrics endpoint.

**Parameters:**
- None

**Authentication:**
- None (access should be restricted at network level)

**Response:**
- Prometheus metrics in text format

**Status Codes:**
- 200: Success

**Example:**
```bash
curl -X GET https://example.com/metrics
```

### Debugging Endpoints

These endpoints are only available when `ENABLE_DEBUG_LOGGING=true`.

#### GET /debug/headers

Shows request headers for debugging.

**Parameters:**
- None

**Authentication:**
- None

**Response:**
- HTML table showing all request headers

**Status Codes:**
- 200: Success
- 404: Not found (when debug mode is disabled)

#### GET /debug/set-token/{username}

Sets a test JWT token in cookies for the specified username.

**Parameters:**
- `username` (path): The username to create a token for

**Authentication:**
- None (development only)

**Response:**
- HTML confirmation of token creation

**Status Codes:**
- 200: Success
- 404: Not found (when debug mode is disabled)

## Error Responses

All API endpoints return errors in a consistent format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "status": 400
}
```

Common error codes:
- `INVALID_INPUT`: Request validation failed
- `NOT_FOUND`: Requested resource not found
- `UNAUTHORIZED`: Missing authentication
- `FORBIDDEN`: Insufficient permissions
- `INTERNAL_ERROR`: Server error

## Rate Limiting

API endpoints are rate-limited to prevent abuse:
- Public endpoints: 100 requests per minute
- Authenticated endpoints: 60 requests per minute
- Failed authentication attempts: 10 per minute

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1595347200
```

## Versioning

This API is currently at version 1 (implicit in the path). Future versions will include version in the path:
```
/v2/api/username/{username}
```
