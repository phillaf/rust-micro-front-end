#!/bin/bash
# Container image backup script
# This script performs a backup of the application container image

set -e # Exit on error

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/images"
APP_IMAGE="rust-micro-front-end:latest"

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR

echo "Starting container image backup at $(date)"

# Save the container image
echo "Saving image $APP_IMAGE..."
docker save $APP_IMAGE | gzip > $BACKUP_DIR/rust-micro-front-end_$TIMESTAMP.tar.gz

echo "Image backup completed: $BACKUP_DIR/rust-micro-front-end_$TIMESTAMP.tar.gz"

echo "Container image backup process completed at $(date)"
