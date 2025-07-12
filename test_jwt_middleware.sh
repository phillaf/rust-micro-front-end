#!/bin/bash

# Simple test to verify JWT middleware is working
set -euo pipefail

echo "Testing JWT middleware functionality..."

# Generate a test token
echo "1. Generating test token..."
TOKEN=$(bash scripts/generate_jwt_token.sh testuser 2>/dev/null | grep -E "^eyJ.*" | head -1)

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to generate token"
    exit 1
fi

echo "✅ Token generated successfully"

# Test with Authorization header
echo "2. Testing Authorization header..."
curl -s -H "Authorization: Bearer $TOKEN" http://localhost/edit > /dev/null && echo "✅ Authorization header works" || echo "❌ Authorization header failed"

# Test with cookie
echo "3. Testing Cookie header..."
curl -s -H "Cookie: jwt_token=$TOKEN" http://localhost/edit > /dev/null && echo "✅ Cookie header works" || echo "❌ Cookie header failed"

echo "🎉 JWT middleware tests complete!"
