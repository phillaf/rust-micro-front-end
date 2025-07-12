#!/bin/bash

# Generate JWT Test Keys
# This script generates RSA key pairs for testing JWT functionality

set -e

echo "Generating JWT test keys..."

# Generate private key
openssl genrsa -out scripts/jwt_private_key.pem 2048

# Generate public key
openssl rsa -in scripts/jwt_private_key.pem -pubout -out scripts/jwt_public_key.pem

echo "JWT keys generated successfully:"
echo "- Private key: scripts/jwt_private_key.pem"
echo "- Public key: scripts/jwt_public_key.pem"

echo ""
echo "Public key content for .env:"
echo "JWT_PUBLIC_KEY=\"$(awk '{printf "%s\\n", $0}' scripts/jwt_public_key.pem)\""

echo ""
echo "Private key content for .env (TESTING ONLY):"
echo "JWT_PRIVATE_KEY=\"$(awk '{printf "%s\\n", $0}' scripts/jwt_private_key.pem)\""
