#!/bin/bash

# Update .env file with JWT keys from generated files
# This script will update the .env file with the JWT keys from the generated files

set -euo pipefail

# Check if the JWT keys exist
if [ ! -f "scripts/jwt_public_key.pem" ] || [ ! -f "scripts/jwt_private_key.pem" ]; then
    echo "JWT keys don't exist. Generating them first..."
    ./scripts/generate_jwt_keys.sh
fi

# Get the JWT public key content
PUBLIC_KEY=$(awk '{printf "%s\\n", $0}' scripts/jwt_public_key.pem)

# Check if the .env file exists
if [ ! -f ".env" ]; then
    echo "Creating .env file..."
    touch .env
fi

# Check if JWT_PUBLIC_KEY already exists in .env
if grep -q "JWT_PUBLIC_KEY=" .env; then
    echo "Updating JWT_PUBLIC_KEY in .env..."
    # Use sed to replace the line
    sed -i "s|JWT_PUBLIC_KEY=.*|JWT_PUBLIC_KEY=$PUBLIC_KEY|" .env
else
    echo "Adding JWT_PUBLIC_KEY to .env..."
    # Append to file
    echo "JWT_PUBLIC_KEY=$PUBLIC_KEY" >> .env
fi

# Set JWT algorithm
if grep -q "JWT_ALGORITHM=" .env; then
    sed -i "s|JWT_ALGORITHM=.*|JWT_ALGORITHM=RS256|" .env
else
    echo "JWT_ALGORITHM=RS256" >> .env
fi

# Set JWT audience
if grep -q "JWT_AUDIENCE=" .env; then
    sed -i "s|JWT_AUDIENCE=.*|JWT_AUDIENCE=micro-frontend-service|" .env
else
    echo "JWT_AUDIENCE=micro-frontend-service" >> .env
fi

# Set JWT issuer
if grep -q "JWT_ISSUER=" .env; then
    sed -i "s|JWT_ISSUER=.*|JWT_ISSUER=test-auth-service|" .env
else
    echo "JWT_ISSUER=test-auth-service" >> .env
fi

echo "JWT environment variables updated successfully in .env file"
