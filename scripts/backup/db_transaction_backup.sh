#!/bin/bash
# Transaction log backup script
# This script performs an incremental backup of database transaction logs

set -e # Exit on error

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/mysql/transactions"
RETENTION_DAYS=7

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR

echo "Starting transaction log backup at $(date)"

# Flush binary logs to ensure consistent backup point
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  -e 'FLUSH BINARY LOGS;'

# Copy binary logs to backup location (this would need adjustment for actual server setup)
# This is a simplified example - in production, you'd need to handle this differently
# depending on where MySQL stores its binary logs
echo "Copying binary logs to backup location"
cp -f /var/lib/mysql/binlog.* $BACKUP_DIR/ 2>/dev/null || echo "No binary logs found - may not be enabled"

echo "Transaction log backup completed at $(date)"

# Clean up old backups based on retention policy
echo "Cleaning up logs older than $RETENTION_DAYS days"
find $BACKUP_DIR -name "binlog.*" -type f -mtime +$RETENTION_DAYS -delete
