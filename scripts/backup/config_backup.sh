#!/bin/bash
# Configuration backup script
# This script performs a backup of application configuration

set -e # Exit on error

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/config"
RETENTION_DAYS=90

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR

echo "Starting configuration backup at $(date)"

# Export environment variables (filtering sensitive data)
env | grep '^APP_' > $BACKUP_DIR/env_config_$TIMESTAMP.txt
env | grep '^DB_' | grep -v 'PASSWORD' >> $BACKUP_DIR/env_config_$TIMESTAMP.txt

echo "Configuration backup completed: $BACKUP_DIR/env_config_$TIMESTAMP.txt"

# Clean up old backups based on retention policy
echo "Cleaning up backups older than $RETENTION_DAYS days"
find $BACKUP_DIR -name "env_config_*.txt" -type f -mtime +$RETENTION_DAYS -delete

echo "Configuration backup process completed at $(date)"
