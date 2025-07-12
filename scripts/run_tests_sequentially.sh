#!/bin/bash

# Run tests sequentially to avoid race conditions with environment variables
# This script runs each test module individually to prevent conflicts

set -e

echo "Running tests sequentially to avoid race conditions..."
echo "======================================================="

cd /home/phil/Projects/rust-micro-front-end

# Run tests by module/category
echo "Running config tests..."
docker compose run --rm app cargo test config::tests --

echo "Running database tests..."
docker compose run --rm app cargo test database::tests --

echo "Running database mock tests..."
docker compose run --rm app cargo test database::mock::tests --

echo "Running middleware tests..."
docker compose run --rm app cargo test middleware::jwt_auth::tests --

echo "All tests completed successfully!"
