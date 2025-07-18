#!/bin/bash
#
# Generate self-signed SSL certificates for development
#

set -e

CERT_DIR="./nginx/certs"
CERT_NAME="server"

# Create certificates directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Generate private key
openssl genrsa -out "$CERT_DIR/$CERT_NAME.key" 2048

# Generate certificate signing request
openssl req -new -key "$CERT_DIR/$CERT_NAME.key" -out "$CERT_DIR/$CERT_NAME.csr" -subj "/CN=localhost/O=Development/C=US"

# Generate self-signed certificate
openssl x509 -req -days 365 -in "$CERT_DIR/$CERT_NAME.csr" -signkey "$CERT_DIR/$CERT_NAME.key" -out "$CERT_DIR/$CERT_NAME.crt"

# Remove CSR as it's no longer needed
rm "$CERT_DIR/$CERT_NAME.csr"

echo "Self-signed SSL certificate generated successfully."
echo "Certificate: $CERT_DIR/$CERT_NAME.crt"
echo "Private Key: $CERT_DIR/$CERT_NAME.key"
echo ""
echo "Note: This is a self-signed certificate for development only."
echo "In production, use a certificate from a trusted CA."
