# Log Aggregation Setup

This document outlines the log aggregation strategy for the Rust Micro Front-End application in a multi-app ecosystem. It covers log collection, processing, and analysis to ensure operational visibility across all services.

## Table of Contents

1. [Log Architecture](#log-architecture)
2. [Log Format Standards](#log-format-standards)
3. [Collection Infrastructure](#collection-infrastructure)
4. [Configuration](#configuration)
5. [Retention Policies](#retention-policies)
6. [Access Control](#access-control)
7. [Integration with Alerting](#integration-with-alerting)
8. [Implementation Guide](#implementation-guide)

## Log Architecture

Our log aggregation architecture follows the modern distributed logging approach:

```ascii
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Micro-App 1    │     │  Micro-App 2    │     │  Micro-App 3    │
│  (This App)     │     │                 │     │                 │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Log Collector (Fluent Bit)                 │
└────────────────────────────────┬──────────────────────────────┬─┘
                                 │                              │
                                 ▼                              ▼
                     ┌─────────────────────┐        ┌─────────────────────┐
                     │  Log Storage        │        │  Log Storage        │
                     │  (Short-term)       │        │  (Long-term)        │
                     │  Elasticsearch      │        │  S3 / GCS / Azure   │
                     └──────────┬──────────┘        └─────────────────────┘
                                │
                                ▼
                     ┌─────────────────────┐
                     │  Visualization      │
                     │  Kibana / Grafana   │
                     └─────────────────────┘
```

## Log Format Standards

All applications in our ecosystem must adhere to the following log format standards:

### JSON Structured Logging

Logs must be emitted in JSON format with the following standard fields:

```json
{
  "timestamp": "2025-07-18T14:30:00.123Z",
  "level": "info",
  "app": "rust-micro-front-end",
  "component": "database",
  "trace_id": "4e7c9630-bcc3-42c4-9fe3-b3b2e6e8a1d0",
  "message": "Database connection established",
  "context": {
    "database": "mysql",
    "host": "db.example.com",
    "connection_pool_size": 10
  }
}
```

### Standard Fields

| Field | Description | Required |
|-------|-------------|----------|
| timestamp | ISO 8601 timestamp with milliseconds | Yes |
| level | Log level (trace, debug, info, warn, error, fatal) | Yes |
| app | Application identifier | Yes |
| component | Component within the application | Yes |
| trace_id | Unique ID for distributed tracing | Yes |
| message | Human-readable log message | Yes |
| context | Object containing additional context | No |

### Log Levels

Standardized log level usage across all applications:

| Level | Usage |
|-------|-------|
| trace | Fine-grained debugging information, highest volume |
| debug | Detailed debugging information |
| info | Normal operation events, service lifecycle events |
| warn | Non-critical issues that should be reviewed |
| error | Error conditions that affect operation but don't halt service |
| fatal | Critical errors that prevent normal operation |

## Collection Infrastructure

### Log Collector

[Fluent Bit](https://fluentbit.io/) is deployed as a sidecar container in each application pod to collect logs:

- Collects logs from stdout/stderr
- Performs initial processing and enrichment
- Buffers logs in case of network issues
- Forwards logs to the central aggregation service

### Aggregation Service

[Elasticsearch](https://www.elastic.co/elasticsearch/) serves as our central log store:

- Indexes logs for fast searching
- Provides full-text search capabilities
- Supports complex queries and aggregations
- Integrates with visualization tools

### Long-term Storage

For compliance and historical analysis, logs are archived to object storage:

- S3-compatible storage (AWS S3, MinIO)
- Lifecycle policies move logs from hot storage to cold storage
- Compressed and encrypted at rest

## Configuration

### Application Configuration

Each application should configure its logger as follows:

```rust
// Example configuration for this application
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::format::JsonFields;
use uuid::Uuid;

pub fn init_logging() {
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "rust-micro-front-end".to_string());
    
    // Generate a unique ID for this process
    let process_id = Uuid::new_v4().to_string();
    
    // Create a layer that logs to stdout in JSON format
    let fmt_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_span_events(fmt::format::FmtSpan::CLOSE)
        .with_target(true);
    
    // Filter logs based on environment variables (RUST_LOG)
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    // Install the tracing subscriber
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    
    // Log application start
    info!(
        app = app_name,
        version = env!("CARGO_PKG_VERSION"),
        process_id = process_id,
        "Application started"
    );
}
```

### Fluent Bit Configuration

Deploy Fluent Bit with this configuration:

```ini
[SERVICE]
    Flush        1
    Log_Level    info
    Parsers_File parsers.conf

[INPUT]
    Name         tail
    Path         /var/log/containers/*.log
    Parser       docker
    Tag          kube.*
    Mem_Buf_Limit  5MB
    Skip_Long_Lines On

[FILTER]
    Name         kubernetes
    Match        kube.*
    Merge_Log    On
    Keep_Log     Off
    K8s-Logging.Parser    On
    K8s-Logging.Exclude   Off

[OUTPUT]
    Name         es
    Match        *
    Host         ${ELASTICSEARCH_HOST}
    Port         ${ELASTICSEARCH_PORT}
    Index        micro-apps
    Type         _doc
    HTTP_User    ${ELASTICSEARCH_USER}
    HTTP_Passwd  ${ELASTICSEARCH_PASSWORD}
    tls          On
    tls.verify   Off
    Retry_Limit  False
    Replace_Dots On
    Logstash_Format On
    Logstash_Prefix micro-apps

[OUTPUT]
    Name          s3
    Match         *
    bucket        ${S3_BUCKET}
    region        ${S3_REGION}
    s3_key_format /logs/%Y/%m/%d/%H/%M/${APP_NAME}_%uuid%.json.gz
    s3_key_format_tag_delimiters .
    use_put_object On
    compression   gzip
    endpoint      ${S3_ENDPOINT}
```

## Retention Policies

| Storage Type | Retention Period | Purpose |
|--------------|-----------------|---------|
| Elasticsearch | 7 days | Active analysis and troubleshooting |
| S3 Warm Storage | 90 days | Recent historical analysis |
| S3 Cold Storage | 1 year | Compliance, security investigations |
| S3 Archive | 7 years | Long-term archival, legal requirements |

## Access Control

### Log Access Roles

| Role | Description | Access Level |
|------|-------------|-------------|
| Operations | DevOps team members | Full access to all logs |
| Developer | Application developers | Read access to their app logs |
| Security | Security team members | Read access to security-related logs |
| Auditor | Compliance team members | Read access to audit logs |

### RBAC Implementation

Implement RBAC through Elasticsearch security features:

```yaml
# Example Elasticsearch role configuration
roles:
  ops_role:
    indices:
      - names: ['micro-apps-*']
        privileges: ['read', 'write', 'manage']
  dev_role:
    indices:
      - names: ['micro-apps-rust-micro-front-end-*']
        privileges: ['read']
        field_security:
          grant: ['*']
          except: ['*.password', '*.secret']
```

## Integration with Alerting

Logs are integrated with alerting systems:

1. **Elasticsearch Alerting** - For pattern-based alerting
2. **Prometheus AlertManager** - For metrics-derived alerts
3. **PagerDuty** - For escalation and on-call notification

### Critical Log Patterns

Configure alerts for these critical log patterns:

| Pattern | Alert Level | Notification |
|---------|-------------|------------|
| `level:error AND component:database` | Warning | Slack + Email |
| `level:fatal` | Critical | PagerDuty |
| `message:*authentication failure* AND count > 5` | Critical | PagerDuty |

## Implementation Guide

### Step 1: Configure Application Logging

Ensure the application uses structured JSON logging as described above. For this application, update `src/logging.rs` as needed.

### Step 2: Deploy Fluent Bit

1. Add the Fluent Bit sidecar to your Kubernetes deployment:

```yaml
# Example Kubernetes configuration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-micro-front-end
spec:
  template:
    spec:
      containers:
        - name: app
          # ...application container config...
        - name: fluent-bit
          image: fluent/fluent-bit:1.9
          volumeMounts:
            - name: fluent-bit-config
              mountPath: /fluent-bit/etc/
            - name: container-logs
              mountPath: /var/log/containers/
              readOnly: true
      volumes:
        - name: fluent-bit-config
          configMap:
            name: fluent-bit-config
        - name: container-logs
          hostPath:
            path: /var/log/containers/
```

1. Create the ConfigMap for Fluent Bit:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluent-bit-config
data:
  fluent-bit.conf: |
    # Fluent Bit configuration from above
  parsers.conf: |
    [PARSER]
        Name        docker
        Format      json
        Time_Key    time
        Time_Format %Y-%m-%dT%H:%M:%S.%L
        Time_Keep   On
```

### Step 3: Configure Elasticsearch

1. Create the appropriate index templates:

```json
PUT _index_template/micro-apps
{
  "index_patterns": ["micro-apps-*"],
  "template": {
    "settings": {
      "number_of_shards": 1,
      "number_of_replicas": 1,
      "index.lifecycle.name": "micro-apps-policy"
    },
    "mappings": {
      "properties": {
        "timestamp": { "type": "date" },
        "level": { "type": "keyword" },
        "app": { "type": "keyword" },
        "component": { "type": "keyword" },
        "trace_id": { "type": "keyword" },
        "message": { "type": "text" }
      }
    }
  }
}
```

1. Create the ILM policy:

```json
PUT _ilm/policy/micro-apps-policy
{
  "policy": {
    "phases": {
      "hot": {
        "min_age": "0ms",
        "actions": {
          "rollover": {
            "max_age": "1d",
            "max_size": "50gb"
          }
        }
      },
      "warm": {
        "min_age": "2d",
        "actions": {
          "shrink": {
            "number_of_shards": 1
          },
          "forcemerge": {
            "max_num_segments": 1
          }
        }
      },
      "cold": {
        "min_age": "7d",
        "actions": {
          "freeze": {}
        }
      },
      "delete": {
        "min_age": "30d",
        "actions": {
          "delete": {}
        }
      }
    }
  }
}
```

### Step 4: Set Up Visualization

1. Create standard Kibana dashboards:
   - Application Overview
   - Error Monitoring
   - User Activity
   - Performance Metrics
   - Security Events

2. Set up saved searches for common queries:
   - All errors in the last 24 hours
   - Authentication failures
   - Slow database queries
   - API endpoint performance

### Step 5: Test Log Collection

1. Generate test logs:

```bash
just dev
# In another terminal
curl http://localhost:8080/health
curl http://localhost:8080/edit
```

1. Verify logs are properly collected in Elasticsearch:

```bash
curl -X GET "http://elasticsearch:9200/micro-apps-*/_search?pretty" -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "app": "rust-micro-front-end"
    }
  },
  "sort": [
    {
      "timestamp": {
        "order": "desc"
      }
    }
  ],
  "size": 20
}
'
```
