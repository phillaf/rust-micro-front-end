#!/bin/bash
# Backup validation script
# This script validates backup and recovery procedures

set -e # Exit on error

echo "Starting backup validation at $(date)"

# Test environment variables
TEST_DB_NAME="validation_test_db"
BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/validation"

# Ensure backup directory exists
mkdir -p $BACKUP_DIR

# Step 1: Test database backup and recovery
echo "Step 1: Testing database backup and recovery..."

# Create a test database
echo "Creating test database $TEST_DB_NAME..."
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  -e "CREATE DATABASE IF NOT EXISTS $TEST_DB_NAME;"

# Create a test table with sample data
echo "Creating test data..."
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  -e "USE $TEST_DB_NAME; 
      CREATE TABLE IF NOT EXISTS validation_test (
          id INT AUTO_INCREMENT PRIMARY KEY,
          name VARCHAR(50) NOT NULL,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
      INSERT INTO validation_test (name) VALUES ('validation_test_1'), ('validation_test_2');"

# Backup the test database
echo "Backing up test database..."
mysqldump \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  $TEST_DB_NAME > $BACKUP_DIR/validation_backup_$BACKUP_TIMESTAMP.sql

# Drop the table to simulate data loss
echo "Simulating data loss by dropping table..."
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  -e "USE $TEST_DB_NAME; DROP TABLE validation_test;"

# Restore the backup
echo "Restoring from backup..."
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  $TEST_DB_NAME < $BACKUP_DIR/validation_backup_$BACKUP_TIMESTAMP.sql

# Verify restoration
echo "Verifying restored data..."
RESULT=$(mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  -e "USE $TEST_DB_NAME; SELECT COUNT(*) FROM validation_test;" -sN)

if [ "$RESULT" -eq 2 ]; then
  echo "✅ Database backup and recovery test successful! ($RESULT records found)"
else
  echo "❌ Database backup and recovery test failed! Expected 2 records, found $RESULT."
  exit 1
fi

# Step 2: Test configuration backup and recovery
echo "Step 2: Testing configuration backup and recovery..."

# Create a test configuration
echo "Creating test configuration..."
TEST_CONFIG_FILE="$BACKUP_DIR/test_config_$BACKUP_TIMESTAMP.txt"
echo "APP_TEST_KEY1=test_value1" > $TEST_CONFIG_FILE
echo "APP_TEST_KEY2=test_value2" >> $TEST_CONFIG_FILE

# Backup the configuration
echo "Backing up test configuration..."
cp $TEST_CONFIG_FILE $BACKUP_DIR/test_config_backup_$BACKUP_TIMESTAMP.txt

# Modify the original configuration
echo "Simulating configuration change..."
echo "APP_TEST_KEY1=modified_value" > $TEST_CONFIG_FILE

# Restore the configuration
echo "Restoring configuration from backup..."
cp $BACKUP_DIR/test_config_backup_$BACKUP_TIMESTAMP.txt $BACKUP_DIR/test_config_restored_$BACKUP_TIMESTAMP.txt

# Verify restoration
echo "Verifying restored configuration..."
DIFF=$(diff $BACKUP_DIR/test_config_backup_$BACKUP_TIMESTAMP.txt $BACKUP_DIR/test_config_restored_$BACKUP_TIMESTAMP.txt)

if [ -z "$DIFF" ]; then
  echo "✅ Configuration backup and recovery test successful!"
else
  echo "❌ Configuration backup and recovery test failed!"
  echo "$DIFF"
  exit 1
fi

# Clean up
echo "Cleaning up test resources..."
mysql \
  --user=$MYSQL_USER \
  --password=$MYSQL_PASSWORD \
  --host=$MYSQL_HOST \
  --port=$MYSQL_PORT \
  -e "DROP DATABASE $TEST_DB_NAME;"

rm -f $BACKUP_DIR/validation_backup_$BACKUP_TIMESTAMP.sql
rm -f $BACKUP_DIR/test_config_$BACKUP_TIMESTAMP.txt
rm -f $BACKUP_DIR/test_config_backup_$BACKUP_TIMESTAMP.txt
rm -f $BACKUP_DIR/test_config_restored_$BACKUP_TIMESTAMP.txt

echo "✅ All backup and recovery tests completed successfully!"
