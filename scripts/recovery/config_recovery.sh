#!/bin/bash
# Configuration recovery script
# This script restores application configuration from backup

set -e # Exit on error

# Parameters
CONFIG_BACKUP=$1

if [ -z "$CONFIG_BACKUP" ]; then
  echo "Usage: $0 <config_backup>"
  echo "Example: $0 env_config_20240710_120000.txt"
  exit 1
fi

BACKUP_DIR="/backup/config"
BACKUP_PATH="$BACKUP_DIR/$CONFIG_BACKUP"

echo "Starting configuration recovery from $CONFIG_BACKUP at $(date)"

# Check if backup file exists
if [ ! -f "$BACKUP_PATH" ]; then
  echo "Error: Backup file $BACKUP_PATH not found"
  exit 1
fi

# Create a temporary .env file from the backup
echo "Restoring environment configuration..."
cp "$BACKUP_PATH" /app/.env.restored

echo "Configuration has been restored to /app/.env.restored"
echo "Review the restored file before applying it to your environment"

echo "Configuration recovery completed at $(date)"
