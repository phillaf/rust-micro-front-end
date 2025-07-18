#!/bin/bash
# Database full backup script
# This script performs a full backup of the MySQL database

set -e # Exit on error

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/mysql/full"
RETENTION_DAYS=30

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR

echo "Starting full database backup at $(date)"

# Run mysqldump with proper options
mysqldump \
  --single-transaction \
  --quick \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  --databases $MYSQL_DATABASE \
  --result-file=$BACKUP_DIR/backup_$TIMESTAMP.sql

# Compress the backup
gzip $BACKUP_DIR/backup_$TIMESTAMP.sql

echo "Database backup completed: $BACKUP_DIR/backup_$TIMESTAMP.sql.gz"

# Clean up old backups based on retention policy
echo "Cleaning up backups older than $RETENTION_DAYS days"
find $BACKUP_DIR -name "backup_*.sql.gz" -type f -mtime +$RETENTION_DAYS -delete

echo "Backup process completed at $(date)"
