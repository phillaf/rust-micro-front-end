# Monitoring Runbooks

This document provides operational runbooks for monitoring and responding to alerts in the Rust Micro Front-End application ecosystem.

## Table of Contents

1. [Alert Response Framework](#alert-response-framework)
2. [High Error Rate Runbook](#high-error-rate-runbook)
3. [Slow Response Time Runbook](#slow-response-time-runbook)
4. [Service Down Runbook](#service-down-runbook)
5. [High Memory Usage Runbook](#high-memory-usage-runbook)
6. [Database Connection Issues Runbook](#database-connection-issues-runbook)
7. [Authentication Failures Runbook](#authentication-failures-runbook)
8. [Incident Reporting Template](#incident-reporting-template)

## Alert Response Framework

### Severity Levels

| Severity | Response Time | Notification | Description |
|----------|---------------|-------------|------------|
| Critical | Immediate (24/7) | PagerDuty, SMS, Email | Service is down or severely degraded for multiple users |
| High | Within 1 hour | PagerDuty, Email | Service is degraded but functional, affecting some users |
| Medium | Within 4 hours | Email | Non-critical component failure, minimal user impact |
| Low | Next business day | Ticket | Minor issue, no user impact |

### Response Workflow

1. **Acknowledge** - Acknowledge the alert in the alerting system
2. **Assess** - Determine severity and impact
3. **Investigate** - Follow the appropriate runbook to diagnose
4. **Mitigate** - Apply immediate mitigation steps
5. **Resolve** - Implement permanent fix
6. **Document** - Complete incident report

## High Error Rate Runbook

**Alert**: `HighErrorRate`

**Description**: HTTP error rate exceeds 5% for 5+ minutes

**Severity**: High

### Investigation Steps

1. **Identify error types**:
   
   ```bash
   # Check error distribution by status code
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=sum(rate(micro_frontend_http_requests_total{status_code=~"5.."}[5m])) by (status_code)' | jq
   ```

2. **Check application logs**:

   ```bash
   # Search for error messages
   curl -s "http://elasticsearch:9200/micro-apps-*/_search" \
     -H 'Content-Type: application/json' \
     -d '{
       "query": {
         "bool": {
           "must": [
             {"match": {"app": "rust-micro-front-end"}},
             {"match": {"level": "error"}},
             {"range": {"timestamp": {"gte": "now-30m"}}}
           ]
         }
       },
       "sort": [{"timestamp": "desc"}],
       "size": 20
     }' | jq
   ```

3. **Check recent code deployments**:

   ```bash
   # Get recent deployments
   kubectl get deployments -o wide
   kubectl rollout history deployment/rust-micro-front-end
   ```

4. **Examine system resources**:

   ```bash
   # Check container resource usage
   kubectl top pods
   ```

### Mitigation Steps

1. **For database-related errors**:
   - Check database connectivity
   - Verify connection pool settings
   - Consider scaling up database resources

2. **For authentication errors**:
   - Verify JWT public key availability
   - Check for expired certificates
   - Ensure auth service is accessible

3. **For resource exhaustion**:
   - Scale up resources
   - Consider horizontal scaling
   
4. **For recent deployment issues**:
   - Roll back to previous version

   ```bash
   kubectl rollout undo deployment/rust-micro-front-end
   ```

### Resolution Verification

1. Monitor error rate metrics to verify resolution:

   ```bash
   # Watch error rate for 5 minutes after mitigation
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=sum(rate(micro_frontend_http_requests_total{status_code=~"5.."}[5m])) / sum(rate(micro_frontend_http_requests_total[5m]))' | jq
   ```

2. Verify impacted endpoints are functioning:

   ```bash
   curl -v http://service-url/health
   curl -v http://service-url/api/username/test
   ```

## Slow Response Time Runbook

**Alert**: `SlowResponses`

**Description**: 95th percentile response time exceeds 2 seconds for 5+ minutes

**Severity**: High

### Investigation Steps

1. **Identify slow endpoints**:

   ```bash
   # Find endpoints with highest latency
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=histogram_quantile(0.95, sum(rate(micro_frontend_http_request_duration_seconds_bucket[5m])) by (le, path))' | jq
   ```

2. **Check database performance**:

   ```bash
   # Look at query duration
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=histogram_quantile(0.95, sum(rate(micro_frontend_database_query_duration_seconds_bucket[5m])) by (le, operation, table))' | jq
   ```

3. **Check system resources**:

   ```bash
   # Check CPU usage
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=rate(process_cpu_seconds_total{app="micro-frontend"}[5m])' | jq
   
   # Check memory usage
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=process_resident_memory_bytes{app="micro-frontend"}' | jq
   ```

4. **Examine connection pools**:

   ```bash
   # Check database connections
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=micro_frontend_database_connections{state="in_use"}' | jq
   ```

### Mitigation Steps

1. **For database bottlenecks**:
   - Check for missing indexes
   - Optimize slow queries
   - Increase connection pool size

2. **For CPU/Memory constraints**:
   - Scale up pod resources
   - Add more replicas

   ```bash
   kubectl scale deployment rust-micro-front-end --replicas=3
   ```

3. **For external service dependencies**:
   - Implement circuit breakers
   - Add timeouts for external calls

4. **For sudden traffic increases**:
   - Implement rate limiting
   - Add caching for common requests

### Resolution Verification

1. Monitor latency metrics to verify improvement:

   ```bash
   # Watch response time for 5 minutes after mitigation
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=histogram_quantile(0.95, sum(rate(micro_frontend_http_request_duration_seconds_bucket[5m])) by (le))' | jq
   ```

2. Perform load testing to verify response times under normal load:

   ```bash
   # Simple load test
   for i in {1..50}; do curl -s -o /dev/null -w "%{time_total}\n" http://service-url/api/username/test; done
   ```

## Service Down Runbook

**Alert**: `InstanceDown`

**Description**: Service instance has been down for 1+ minutes

**Severity**: Critical

### Investigation Steps

1. **Check pod status**:

   ```bash
   kubectl get pods
   kubectl describe pod <pod-name>
   kubectl logs <pod-name> --previous
   ```

2. **Check for OOM (Out of Memory) events**:

   ```bash
   kubectl describe pod <pod-name> | grep -i "OOMKilled"
   ```

3. **Check for node issues**:

   ```bash
   kubectl get nodes
   kubectl describe node <node-name>
   ```

4. **Verify network connectivity**:

   ```bash
   kubectl exec <some-pod> -- ping <service-ip>
   kubectl exec <some-pod> -- curl -v http://<service-name>:<port>/health
   ```

### Mitigation Steps

1. **For crashed pods**:
   - Restart the pod:

   ```bash
   kubectl delete pod <pod-name>
   ```

2. **For resource issues**:
   - Scale up resources:

   ```bash
   kubectl edit deployment rust-micro-front-end
   # Increase resources in the editor
   ```

3. **For node failures**:
   - Cordon failing node and reschedule pods:

   ```bash
   kubectl cordon <node-name>
   kubectl drain <node-name> --ignore-daemonsets --delete-local-data
   ```

4. **For persistent failures**:
   - Roll back to last known good deployment:

   ```bash
   kubectl rollout history deployment/rust-micro-front-end
   kubectl rollout undo deployment/rust-micro-front-end --to-revision=<revision>
   ```

### Resolution Verification

1. Verify pod is running:

   ```bash
   kubectl get pods | grep rust-micro-front-end
   ```

2. Check health endpoint:

   ```bash
   curl -v http://service-url/health
   ```

3. Monitor uptime metric:

   ```bash
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=up{app="micro-frontend"}' | jq
   ```

## High Memory Usage Runbook

**Alert**: `HighMemoryUsage`

**Description**: Memory usage exceeds 500MB for 15+ minutes

**Severity**: Medium

### Investigation Steps

1. **Analyze memory trends**:

   ```bash
   # Check memory growth over time
   curl -s "http://prometheus:9090/api/v1/query_range" \
     --data-urlencode 'query=process_resident_memory_bytes{app="micro-frontend"}/(1024*1024)' \
     --data-urlencode 'start=1h' \
     --data-urlencode 'end=now' \
     --data-urlencode 'step=5m' | jq
   ```

2. **Check for memory leaks**:

   ```bash
   # If memory continuously increases without plateauing, suspect a leak
   # Look at heap dumps if available
   ```

3. **Examine connection pools and caches**:

   ```bash
   # Database connections
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=micro_frontend_database_connections' | jq
   
   # Cache size metrics if available
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=micro_frontend_cache_size_bytes' | jq
   ```

4. **Check resource limits**:

   ```bash
   kubectl describe pod <pod-name> | grep -A 5 "Limits:"
   ```

### Mitigation Steps

1. **For memory leaks**:
   - Restart affected pods as temporary fix:

   ```bash
   kubectl delete pod <pod-name>
   ```

2. **For legitimate high usage**:
   - Scale up memory limits:

   ```bash
   kubectl edit deployment rust-micro-front-end
   # Increase memory resources in the editor
   ```

3. **For cache-related issues**:
   - Adjust cache TTL or size limits
   - Implement memory-bound caching strategy

4. **For connection pool issues**:
   - Ensure connections are properly closed
   - Adjust connection pool size

### Resolution Verification

1. Monitor memory usage after mitigation:

   ```bash
   # Watch memory usage for 15 minutes
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=process_resident_memory_bytes{app="micro-frontend"}/(1024*1024)' | jq
   ```

2. Verify application functionality:

   ```bash
   # Check key endpoints
   curl -v http://service-url/health
   curl -v http://service-url/display?username=test
   ```

## Database Connection Issues Runbook

**Alert**: `DatabaseConnectionErrors` or high rate of database errors

**Description**: Multiple database connection errors in short time period

**Severity**: High

### Investigation Steps

1. **Check database connectivity**:

   ```bash
   # Check database connection errors
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=rate(micro_frontend_database_connection_errors_total[5m])' | jq
   ```

2. **Verify database health**:

   ```bash
   # For MySQL
   kubectl exec -it <mysql-pod> -- mysql -u <user> -p -e "SHOW GLOBAL STATUS LIKE '%Connections%';"
   kubectl exec -it <mysql-pod> -- mysql -u <user> -p -e "SHOW PROCESSLIST;"
   ```

3. **Check connection pool settings**:

   ```bash
   # Review connection pool metrics
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=micro_frontend_database_connections' | jq
   ```

4. **Examine network policies**:

   ```bash
   kubectl get networkpolicies
   ```

### Mitigation Steps

1. **For connection pool exhaustion**:
   - Increase pool size in configuration
   - Reduce query duration/frequency

2. **For database overload**:
   - Scale up database resources
   - Implement read replicas for read-heavy operations

3. **For network issues**:
   - Verify network policies allow proper connectivity
   - Check DNS resolution

4. **For credential issues**:
   - Verify credentials are correct in secrets
   - Rotate credentials if compromised

### Resolution Verification

1. Monitor database connection errors:

   ```bash
   # Watch connection errors for 5 minutes
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=rate(micro_frontend_database_connection_errors_total[5m])' | jq
   ```

2. Verify query performance:

   ```bash
   # Check query durations
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=histogram_quantile(0.95, sum(rate(micro_frontend_database_query_duration_seconds_bucket[5m])) by (le))' | jq
   ```

## Authentication Failures Runbook

**Alert**: `HighAuthFailureRate`

**Description**: Authentication failure rate exceeds 20% over 5+ minutes

**Severity**: High (potential security incident)

### Investigation Steps

1. **Analyze auth failure patterns**:

   ```bash
   # Check authentication failure rate
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=sum(rate(micro_frontend_auth_attempts_total{status="failure"}[5m])) / sum(rate(micro_frontend_auth_attempts_total[5m]))' | jq
   ```

2. **Check JWT validation issues in logs**:

   ```bash
   # Search for JWT errors
   curl -s "http://elasticsearch:9200/micro-apps-*/_search" \
     -H 'Content-Type: application/json' \
     -d '{
       "query": {
         "bool": {
           "must": [
             {"match": {"app": "rust-micro-front-end"}},
             {"match_phrase": {"message": "JWT validation"}}
           ]
         }
       },
       "sort": [{"timestamp": "desc"}],
       "size": 20
     }' | jq
   ```

3. **Verify JWT public key**:

   ```bash
   # Check public key is accessible
   kubectl exec <pod-name> -- cat /path/to/jwt_public_key.pem | head -3
   ```

4. **Check for potential attacks**:

   ```bash
   # Look for patterns suggesting brute force
   curl -s "http://elasticsearch:9200/micro-apps-*/_search" \
     -H 'Content-Type: application/json' \
     -d '{
       "aggs": {
         "ips": {
           "terms": {
             "field": "client_ip",
             "size": 10,
             "order": {
               "_count": "desc"
             }
           }
         }
       },
       "query": {
         "bool": {
           "must": [
             {"match": {"app": "rust-micro-front-end"}},
             {"match": {"level": "warn"}},
             {"match_phrase": {"message": "authentication failure"}}
           ]
         }
       },
       "size": 0
     }' | jq
   ```

### Mitigation Steps

1. **For JWT key issues**:
   - Verify public key is correctly mounted
   - Check key expiration
   - Ensure key formatting is correct

2. **For potential attacks**:
   - Implement rate limiting for authentication attempts
   - Temporarily block suspicious IPs
   
   ```bash
   # Example of blocking an IP with iptables
   kubectl exec <proxy-pod> -- iptables -A INPUT -s <suspicious-ip> -j DROP
   ```

3. **For auth service issues**:
   - Check connectivity to auth service
   - Verify auth service health

4. **For clock skew issues**:
   - Verify server times are synchronized
   - Adjust JWT validation leeway

### Resolution Verification

1. Monitor authentication success rate:

   ```bash
   # Watch auth success rate for 5 minutes
   curl -s "http://prometheus:9090/api/v1/query" \
     --data-urlencode 'query=sum(rate(micro_frontend_auth_attempts_total{status="success"}[5m])) / sum(rate(micro_frontend_auth_attempts_total[5m]))' | jq
   ```

2. Test authentication with valid credentials:

   ```bash
   # Get a test token
   TOKEN=$(./scripts/jwt_test_helper.sh testuser)
   
   # Test authentication
   curl -v -H "Authorization: Bearer $TOKEN" http://service-url/edit
   ```

## Incident Reporting Template

After resolving an incident, complete this incident report:

```
# Incident Report

## Summary
- **Incident ID**: INC-YYYY-MM-DD-XX
- **Date/Time**: YYYY-MM-DD HH:MM UTC
- **Duration**: X hours, Y minutes
- **Severity**: Critical/High/Medium/Low
- **Service(s) Affected**: rust-micro-front-end
- **Impact**: Brief description of user impact

## Timeline
- **HH:MM** - Alert triggered
- **HH:MM** - Investigation started
- **HH:MM** - Root cause identified
- **HH:MM** - Mitigation applied
- **HH:MM** - Service restored
- **HH:MM** - Incident closed

## Root Cause
Detailed explanation of what caused the incident.

## Resolution
Steps taken to resolve the incident.

## Prevention
Actions to prevent similar incidents in the future.

## Lessons Learned
What did we learn from this incident?

## Action Items
- [ ] Action 1 (Owner: Name, Due: YYYY-MM-DD)
- [ ] Action 2 (Owner: Name, Due: YYYY-MM-DD)

```
