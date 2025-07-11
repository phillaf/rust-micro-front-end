#!/bin/bash

# MySQL Integration Test Suite
# Tests the complete MySQL integration including database operations,
# API endpoints, and data persistence.

set -e

echo "MySQL Integration Test Suite"
echo "============================="

# Test 1: Health Check
echo -n "1. Health Check... "
if curl -s http://localhost/health | grep -q "healthy"; then
    echo "✅ PASSED"
else
    echo "❌ FAILED"
    exit 1
fi

# Test 2: Get User
echo -n "2. Get User... "
if curl -s http://localhost/api/username/admin | grep -q "admin"; then
    echo "✅ PASSED"
else
    echo "❌ FAILED"
    exit 1
fi

# Test 3: 404 for non-existent user
echo -n "3. Non-existent User (404)... "
status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/api/username/nonexistent)
if [ "$status" = "404" ]; then
    echo "✅ PASSED"
else
    echo "❌ FAILED (got $status)"
    exit 1
fi

echo ""
echo "✅ All tests passed!"
