#!/bin/bash

# MySQL Integration Test Script
# This script tests the full MySQL integration of the application

echo "Testing MySQL Integration..."
echo "=================================="

# Test 1: Health Check (should connect to MySQL and return user count)
echo "1. Testing health check endpoint..."
curl -s http://localhost/health | jq '.' || echo "Health check failed"
echo ""

# Test 2: Get existing user data (should retrieve from MySQL)
echo "2. Testing username endpoint for existing user 'admin'..."
curl -s http://localhost/api/username/admin | jq '.' || echo "Username retrieval failed"
echo ""

# Test 3: Get non-existing user (should return 404)
echo "3. Testing username endpoint for non-existing user..."
curl -s http://localhost/api/username/nonexistent | jq '.' || echo "Expected 404 for non-existing user"
echo ""

# Test 4: Update existing user display name (should update in MySQL)
echo "4. Testing update username endpoint..."
curl -s -X POST http://localhost/api/username \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "display_name": "Updated Administrator"}' | jq '.' || echo "Update failed"
echo ""

# Test 5: Verify the update was persisted in MySQL
echo "5. Verifying update was persisted..."
curl -s http://localhost/api/username/admin | jq '.' || echo "Verification failed"
echo ""

# Test 6: Create new user (should insert into MySQL)
echo "6. Testing creation of new user..."
curl -s -X POST http://localhost/api/username \
  -H "Content-Type: application/json" \
  -d '{"username": "newuser", "display_name": "New User Display Name"}' | jq '.' || echo "Creation failed"
echo ""

# Test 7: Verify new user was created
echo "7. Verifying new user was created..."
curl -s http://localhost/api/username/newuser | jq '.' || echo "New user verification failed"
echo ""

echo "MySQL Integration Test Complete!"
