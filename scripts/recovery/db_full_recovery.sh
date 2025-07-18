#!/bin/bash
# Database full recovery script
# This script performs a recovery from a full database backup

set -e # Exit on error

# Parameters
BACKUP_FILE=$1

if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup_file>"
  echo "Example: $0 backup_20240710_120000.sql.gz"
  exit 1
fi

BACKUP_DIR="/backup/mysql/full"
BACKUP_PATH="$BACKUP_DIR/$BACKUP_FILE"

echo "Starting database recovery from $BACKUP_FILE at $(date)"

# Check if backup file exists
if [ ! -f "$BACKUP_PATH" ]; then
  echo "Error: Backup file $BACKUP_PATH not found"
  exit 1
fi

# Decompress if needed
if [[ "$BACKUP_FILE" == *.gz ]]; then
  echo "Decompressing backup file..."
  gunzip -c "$BACKUP_PATH" > "${BACKUP_PATH%.gz}"
  BACKUP_FILE="${BACKUP_FILE%.gz}"
  BACKUP_PATH="${BACKUP_PATH%.gz}"
fi

# Restore the database
echo "Restoring database from $BACKUP_PATH..."
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  < "$BACKUP_PATH"

echo "Database recovery completed at $(date)"
