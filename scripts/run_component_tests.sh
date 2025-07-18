#!/bin/bash
# Component isolation test runner

set -e

echo "Setting up test environment..."

# Install necessary dependencies if not already installed
if ! npm list jsdom &>/dev/null; then
    npm install --no-save jsdom node-fetch jest @babel/core @babel/preset-env
fi

# Set environment variables for testing
export TEST_SERVER_URL=${TEST_SERVER_URL:-"http://localhost:3000"}
export NODE_OPTIONS="--unhandled-rejections=strict"

# Generate a test JWT token if needed
if [ -z "$TEST_JWT_TOKEN" ]; then
    echo "Generating test JWT token..."
    if [ -f "/app/scripts/jwt_test_helper.sh" ]; then
        export TEST_JWT_TOKEN=$(bash /app/scripts/jwt_test_helper.sh generate test_user)
    else
        echo "Warning: jwt_test_helper.sh not found, tests requiring JWT may fail"
        export TEST_JWT_TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJpc3MiOiJ0ZXN0X2lzc3VlciIsImF1ZCI6InRlc3RfYXVkaWVuY2UiLCJleHAiOjk5OTk5OTk5OTl9.signature"
    fi
fi

# Create Jest config
cat > jest.config.js << EOL
module.exports = {
  testEnvironment: 'node',
  transform: {
    '^.+\\.js$': 'babel-jest',
  },
  transformIgnorePatterns: [
    '/node_modules/'
  ],
  verbose: true,
  bail: 0,
  testTimeout: 10000
};
EOL

# Create Babel config
cat > babel.config.js << EOL
module.exports = {
  presets: [
    [
      '@babel/preset-env',
      {
        targets: {
          node: 'current',
        },
      },
    ],
  ],
};
EOL

# Run tests
echo "Running component isolation tests..."
jest tests/component/ --runInBand

# Clean up temporary files
rm -f jest.config.js babel.config.js
