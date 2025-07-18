#!/bin/bash
# Container image recovery script
# This script restores a container image from backup

set -e # Exit on error

# Parameters
IMAGE_BACKUP=$1

if [ -z "$IMAGE_BACKUP" ]; then
  echo "Usage: $0 <image_backup>"
  echo "Example: $0 rust-micro-front-end_20240710_120000.tar.gz"
  exit 1
fi

BACKUP_DIR="/backup/images"
BACKUP_PATH="$BACKUP_DIR/$IMAGE_BACKUP"

echo "Starting image recovery from $IMAGE_BACKUP at $(date)"

# Check if backup file exists
if [ ! -f "$BACKUP_PATH" ]; then
  echo "Error: Backup file $BACKUP_PATH not found"
  exit 1
fi

# Load the image
echo "Loading image from backup..."
docker load < "$BACKUP_PATH"

echo "Image recovery completed at $(date)"
