# Backup and Recovery Procedures

This document outlines the backup and recovery procedures for the Rust Micro Front-End application, ensuring data integrity and service continuity in case of failures.

## Table of Contents

1. [Backup Strategy Overview](#backup-strategy-overview)
2. [Data Assets Inventory](#data-assets-inventory)
3. [Backup Procedures](#backup-procedures)
4. [Recovery Procedures](#recovery-procedures)
5. [Testing and Validation](#testing-and-validation)
6. [Security Considerations](#security-considerations)
7. [Implementation Guide](#implementation-guide)

## Backup Strategy Overview

Our backup strategy follows the 3-2-1 principle:

- 3 copies of data
- 2 different storage media types
- 1 copy offsite

### Backup Types

| Type | Frequency | Retention | Purpose |
|------|-----------|-----------|---------|
| Full Database | Daily | 30 days | Complete point-in-time recovery |
| Transaction Logs | Hourly | 7 days | Point-in-time recovery within the last week |
| Configuration | On change | 90 days | Infrastructure recovery |
| Container Images | On release | Indefinite | Application recovery |

## Data Assets Inventory

### Critical Data Assets

| Asset | Type | Backup Method | Recovery Time Objective | Recovery Point Objective |
|-------|------|--------------|-------------------------|--------------------------|
| MySQL Database | Structured Data | Automated mysqldump + Binary Logs | 30 minutes | 1 hour |
| Environment Configuration | Configuration | Git-versioned + Secure Storage | 15 minutes | 0 (no data loss) |
| JWT Public Keys | Security Assets | Git-versioned + Secure Storage | 15 minutes | 0 (no data loss) |
| Application Logs | Operational Data | Log Shipping + S3 Archive | 1 hour | 1 hour |
| Container Images | Application | Registry Replication | 15 minutes | 0 (no data loss) |

## Backup Procedures

### Database Backup

#### Daily Full Backup

```bash
#!/bin/bash
# File: /scripts/backup/db_full_backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/mysql/full"
RETENTION_DAYS=30

# Ensure backup directory exists
mkdir -p $BACKUP_DIR

# Run mysqldump with proper options
just run-backup-command "mysqldump \
  --single-transaction \
  --quick \
  --user=\$MYSQL_USER \
  --password=\$MYSQL_PASSWORD \
  --host=\$MYSQL_HOST \
  --port=\$MYSQL_PORT \
  --databases \$MYSQL_DATABASE \
  --result-file=/backup/mysql/full/backup_\$TIMESTAMP.sql"

# Compress the backup
just run-backup-command "gzip /backup/mysql/full/backup_\$TIMESTAMP.sql"

# Upload to secure storage
just run-backup-command "aws s3 cp /backup/mysql/full/backup_\$TIMESTAMP.sql.gz s3://\$BACKUP_BUCKET/mysql/full/"

# Clean up old backups
find $BACKUP_DIR -name "backup_*.sql.gz" -type f -mtime +$RETENTION_DAYS -delete
```

#### Hourly Transaction Log Backup

```bash
#!/bin/bash
# File: /scripts/backup/db_transaction_backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/mysql/transactions"
RETENTION_DAYS=7

# Ensure backup directory exists
mkdir -p $BACKUP_DIR

# Flush and copy binary logs
just run-backup-command "mysql \
  --user=\$MYSQL_USER \
  --password=\$MYSQL_PASSWORD \
  --host=\$MYSQL_HOST \
  --port=\$MYSQL_PORT \
  -e 'FLUSH BINARY LOGS;'"

# Copy the binary logs to backup location
just run-backup-command "cp /var/lib/mysql/binlog.* $BACKUP_DIR/"

# Upload to secure storage
just run-backup-command "aws s3 sync $BACKUP_DIR s3://\$BACKUP_BUCKET/mysql/transactions/"

# Clean up old backups
find $BACKUP_DIR -name "binlog.*" -type f -mtime +$RETENTION_DAYS -delete
```

### Configuration Backup

Configuration is stored in two locations:

1. Git repository (versioned changes)
2. Secure vault (sensitive information)

```bash
#!/bin/bash
# File: /scripts/backup/config_backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/config"
RETENTION_DAYS=90

# Ensure backup directory exists
mkdir -p $BACKUP_DIR

# Export configurations from secure vault
just run-backup-command "vault kv get -format=json secret/rust-micro-front-end > $BACKUP_DIR/vault_config_$TIMESTAMP.json"

# Export environment variables
just run-backup-command "env | grep '^APP_' > $BACKUP_DIR/env_config_$TIMESTAMP.txt"
just run-backup-command "env | grep '^DB_' >> $BACKUP_DIR/env_config_$TIMESTAMP.txt"

# Upload to secure storage
just run-backup-command "aws s3 cp $BACKUP_DIR/vault_config_$TIMESTAMP.json s3://\$BACKUP_BUCKET/config/"
just run-backup-command "aws s3 cp $BACKUP_DIR/env_config_$TIMESTAMP.txt s3://\$BACKUP_BUCKET/config/"

# Clean up old backups
find $BACKUP_DIR -name "vault_config_*.json" -type f -mtime +$RETENTION_DAYS -delete
find $BACKUP_DIR -name "env_config_*.txt" -type f -mtime +$RETENTION_DAYS -delete
```

### Container Image Backup

Container images are backed up by:

1. Using a multi-region container registry
2. Periodic exports of container images

```bash
#!/bin/bash
# File: /scripts/backup/image_backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/images"

# Ensure backup directory exists
mkdir -p $BACKUP_DIR

# Save container image
just run-backup-command "docker save rust-micro-front-end:latest | gzip > $BACKUP_DIR/rust-micro-front-end_$TIMESTAMP.tar.gz"

# Upload to secure storage
just run-backup-command "aws s3 cp $BACKUP_DIR/rust-micro-front-end_$TIMESTAMP.tar.gz s3://\$BACKUP_BUCKET/images/"
```

## Recovery Procedures

### Database Recovery

#### Full Database Recovery

```bash
#!/bin/bash
# File: /scripts/recovery/db_full_recovery.sh

# Parameters
BACKUP_FILE=$1 # e.g., backup_20240710_120000.sql.gz

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup_file>"
  exit 1
fi

# Download from secure storage if needed
if [[ ! -f "/backup/mysql/full/$BACKUP_FILE" ]]; then
  just run-recovery-command "aws s3 cp s3://\$BACKUP_BUCKET/mysql/full/$BACKUP_FILE /backup/mysql/full/"
fi

# Decompress if needed
if [[ "$BACKUP_FILE" == *.gz ]]; then
  just run-recovery-command "gunzip -c /backup/mysql/full/$BACKUP_FILE > /backup/mysql/full/${BACKUP_FILE%.gz}"
  BACKUP_FILE="${BACKUP_FILE%.gz}"
fi

# Restore the database
just run-recovery-command "mysql \
  --user=\$MYSQL_USER \
  --password=\$MYSQL_PASSWORD \
  --host=\$MYSQL_HOST \
  --port=\$MYSQL_PORT \
  < /backup/mysql/full/$BACKUP_FILE"

echo "Database restored from $BACKUP_FILE"
```

#### Point-in-Time Recovery

```bash
#!/bin/bash
# File: /scripts/recovery/db_point_in_time_recovery.sh

# Parameters
BACKUP_FILE=$1 # e.g., backup_20240710_120000.sql.gz
RECOVERY_TIMESTAMP=$2 # e.g., "2024-07-10 15:30:00"

if [ -z "$BACKUP_FILE" ] || [ -z "$RECOVERY_TIMESTAMP" ]; then
  echo "Usage: $0 <backup_file> <recovery_timestamp>"
  exit 1
fi

# First perform full recovery
/scripts/recovery/db_full_recovery.sh "$BACKUP_FILE"

# Apply binary logs up to the specified time
just run-recovery-command "mysqlbinlog \
  --stop-datetime=\"$RECOVERY_TIMESTAMP\" \
  /backup/mysql/transactions/binlog.* | \
  mysql \
  --user=\$MYSQL_USER \
  --password=\$MYSQL_PASSWORD \
  --host=\$MYSQL_HOST \
  --port=\$MYSQL_PORT"

echo "Database restored to point in time: $RECOVERY_TIMESTAMP"
```

### Configuration Recovery

```bash
#!/bin/bash
# File: /scripts/recovery/config_recovery.sh

# Parameters
CONFIG_BACKUP=$1 # e.g., vault_config_20240710_120000.json

if [ -z "$CONFIG_BACKUP" ]; then
  echo "Usage: $0 <config_backup>"
  exit 1
fi

# Download from secure storage if needed
if [[ ! -f "/backup/config/$CONFIG_BACKUP" ]]; then
  just run-recovery-command "aws s3 cp s3://\$BACKUP_BUCKET/config/$CONFIG_BACKUP /backup/config/"
fi

# Restore configuration to vault
just run-recovery-command "cat /backup/config/$CONFIG_BACKUP | vault kv put secret/rust-micro-front-end -"

echo "Configuration restored from $CONFIG_BACKUP"
```

### Container Image Recovery

```bash
#!/bin/bash
# File: /scripts/recovery/image_recovery.sh

# Parameters
IMAGE_BACKUP=$1 # e.g., rust-micro-front-end_20240710_120000.tar.gz

if [ -z "$IMAGE_BACKUP" ]; then
  echo "Usage: $0 <image_backup>"
  exit 1
fi

# Download from secure storage if needed
if [[ ! -f "/backup/images/$IMAGE_BACKUP" ]]; then
  just run-recovery-command "aws s3 cp s3://\$BACKUP_BUCKET/images/$IMAGE_BACKUP /backup/images/"
fi

# Load the image
just run-recovery-command "gunzip -c /backup/images/$IMAGE_BACKUP | docker load"

echo "Container image restored from $IMAGE_BACKUP"
```

## Testing and Validation

### Backup Testing Schedule

| Backup Type | Testing Frequency | Validation Method |
|-------------|------------------|-------------------|
| Full Database | Monthly | Restore to test environment |
| Transaction Logs | Quarterly | Point-in-time recovery test |
| Configuration | Quarterly | Apply to test environment |
| Container Images | On each release | Deploy to test environment |

### Validation Procedure

```bash
#!/bin/bash
# File: /scripts/validation/backup_validation.sh

# This script should be run in the test environment

# Test database restoration
echo "Testing database restoration..."
/scripts/recovery/db_full_recovery.sh "latest_backup.sql.gz"

# Validate data integrity
just run-validation-command "mysql \
  --user=\$MYSQL_USER \
  --password=\$MYSQL_PASSWORD \
  --host=\$MYSQL_HOST \
  --port=\$MYSQL_PORT \
  -e 'SELECT COUNT(*) FROM users;'"

# Test configuration restoration
echo "Testing configuration restoration..."
/scripts/recovery/config_recovery.sh "latest_config.json"

# Test container image restoration
echo "Testing container image restoration..."
/scripts/recovery/image_recovery.sh "latest_image.tar.gz"

echo "Validation complete"
```

## Security Considerations

### Backup Security

- All backups are encrypted at rest using AES-256
- Transport encryption using TLS 1.3
- Access control using IAM roles with least privilege principle
- Regular rotation of backup encryption keys

### Recovery Security

- Multi-factor authentication required for recovery procedures
- Audit logging of all recovery operations
- Separate recovery credentials from regular operational credentials
- Post-recovery security validation

## Implementation Guide

### Adding to Justfile

```makefile
# Add these to justfile

# Backup commands
backup-database:
    @echo "Backing up database..."
    @docker compose exec app /scripts/backup/db_full_backup.sh

backup-config:
    @echo "Backing up configuration..."
    @docker compose exec app /scripts/backup/config_backup.sh

backup-images:
    @echo "Backing up container images..."
    @/scripts/backup/image_backup.sh

backup-all: backup-database backup-config backup-images
    @echo "All backups completed"

# Recovery commands
recover-database BACKUP_FILE:
    @echo "Recovering database from {{BACKUP_FILE}}..."
    @docker compose exec app /scripts/recovery/db_full_recovery.sh {{BACKUP_FILE}}

recover-database-point-in-time BACKUP_FILE TIMESTAMP:
    @echo "Recovering database to {{TIMESTAMP}}..."
    @docker compose exec app /scripts/recovery/db_point_in_time_recovery.sh {{BACKUP_FILE}} "{{TIMESTAMP}}"

recover-config CONFIG_BACKUP:
    @echo "Recovering configuration from {{CONFIG_BACKUP}}..."
    @docker compose exec app /scripts/recovery/config_recovery.sh {{CONFIG_BACKUP}}

recover-image IMAGE_BACKUP:
    @echo "Recovering image from {{IMAGE_BACKUP}}..."
    @/scripts/recovery/image_recovery.sh {{IMAGE_BACKUP}}

# Validation commands
validate-backups:
    @echo "Validating backups..."
    @docker compose -f compose.test.yml exec app /scripts/validation/backup_validation.sh
```

### Setting Up Automation

1. Create the backup scripts in `/scripts/backup/`
2. Set up cron jobs to run the backups according to the schedule
3. Configure secure storage (S3, Azure Blob, etc.)
4. Test the recovery procedures in a staging environment
5. Document the results and update this guide as needed
