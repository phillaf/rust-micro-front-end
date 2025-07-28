# Rust Micro Front-End Development Commands
# This justfile provides containerized development commands

# Get current user and group IDs to avoid permission issues
export UID := `id -u`
export GID := `id -g`

# Default recipe lists available commands
default:
    @just --list

# ===== DEVELOPMENT WORKFLOW =====

# Start an interactive shell in the development container
bash:
    docker compose --profile dev run -it --rm app bash

# Build the application in development mode
build:
    docker compose --profile dev run --rm app cargo build

# Run the development server
# Note: This must be started manually due to tool constraints
dev:
    docker compose --profile dev run --rm --service-ports app cargo run --bin rust-micro-front-end

# Check the code for compilation errors
check:
    docker compose --profile dev run --rm app cargo check

# Format code using rustfmt
format:
    docker compose --profile dev run --rm app cargo fmt

# Lint code using clippy
lint:
    docker compose --profile dev run --rm app cargo clippy

# Clean build artifacts
clean:
    docker compose --profile dev run --rm app cargo clean

# Fix permission issues with target directory
fix-permissions:
    @echo "Fixing permissions on target directory..."
    sudo chown -R ${UID}:${GID} target/ || { echo "Failed to fix permissions"; exit 1; }

# Initialize cargo cache with correct permissions
init-cargo-cache:
    @echo "Initializing cargo caches with correct permissions..."
    docker compose --profile dev run --rm --user root app chown -R ${UID}:${GID} /usr/local/cargo || true
    docker compose --profile dev run --rm --user root app chown -R ${UID}:${GID} /usr/local/rustup || true

# Nuclear clean - removes target directory entirely and recreates with correct permissions
clean-nuclear:
    @echo "Performing nuclear clean..."
    sudo rm -rf target/
    just init-cargo-cache
    docker compose --profile dev run --rm app cargo check > /dev/null

# ===== DATABASE OPERATIONS =====

# Start the development database
db-up:
    @echo "Starting MySQL database..."
    docker compose up -d mysql || { echo "Failed to start MySQL"; exit 1; }
    @echo "Waiting for MySQL to be ready..."
    docker compose exec mysql bash -c 'until mysqladmin ping -h localhost --silent; do sleep 1; done' || { echo "MySQL startup failed"; exit 1; }
    @echo "MySQL database is ready!"

# Run database migrations
migrate:
    @echo "Running database migrations..."
    docker compose --profile dev run --rm app cargo run --bin migrate || { echo "Migration failed"; exit 1; }

# Access database shell
db-shell:
    @echo "Accessing database shell..."
    docker compose exec mysql bash -c 'mysql -u "$MYSQL_USER" -p"$MYSQL_PASSWORD" "$MYSQL_DATABASE"'

# Check if database is seeded
db-check-seeding:
    @echo "Checking database seeding status..."
    docker compose --profile dev run --rm app cargo run --bin check_seeding

# ===== TESTING =====

# Run all tests
test:
    @echo "Running test suite in containers..."
    docker compose --profile dev run --rm app cargo test -- --test-threads=1 || { echo "Test execution failed"; exit 1; }

# Run all tests in parallel (faster but may have race conditions)
test-parallel:
    @echo "Running test suite in parallel..."
    docker compose --profile dev run --rm app cargo test

# Run a specific test module
test-module module_name:
    @echo "Running test module: {{module_name}}..."
    docker compose --profile dev run --rm app cargo test {{module_name}} -- --test-threads=1

# Test JWT authentication with helper script
test-jwt-auth:
    @echo "Running JWT authentication tests with helper script..."
    docker compose --profile dev run --rm app /usr/src/myapp/scripts/jwt_test_helper.sh

# Reset JWT authentication - provides consistent JWT tokens and debug link
auth-reset username="testuser":
    @echo "🔑 Resetting JWT authentication for user: {{username}}..."
    docker compose --profile dev run --rm app /usr/src/myapp/scripts/jwt_test_helper.sh {{username}}
    @echo -e "\n✅ Use the URL above to set the JWT token, then you can access:"
    @echo "   - http://localhost/edit"
    @echo "   - POST to http://localhost/api/username"
    @echo "   - Other JWT-protected endpoints"

# Check JWT token validity and inspect contents
jwt-check token="":
    @echo "🔍 Checking JWT token validity..."
    @if [ -n "{{token}}" ]; then \
        ./scripts/check_jwt.sh "{{token}}"; \
    else \
        ./scripts/check_jwt.sh; \
    fi

# Check container user and environment
check-container-user:
    @echo "Checking container user and environment..."
    docker compose --profile dev run --rm app bash -c "id && whoami && cat /etc/passwd | grep developer && echo 'ENV: JWT_PUBLIC_KEY exists: ' && [ -n \"$$JWT_PUBLIC_KEY\" ] && echo 'Yes' || echo 'No' && env | grep JWT"

# Set up JWT environment
setup-jwt:
    @echo "Setting up JWT environment..."
    ./scripts/generate_jwt_keys.sh
    ./scripts/update_jwt_env.sh

# Complete environment setup
setup:
    @echo "Setting up complete development environment..."
    just setup-jwt
    just init-cargo-cache
    @echo "Building Docker containers..."
    docker compose --profile dev build
    @echo "Starting database..."
    just db-up
    @echo "Running migrations..."
    just migrate
    @echo "Setup complete! You can now run 'just dev' to start the development server."

# Run component tests
test-components:
    @echo "Running component tests..."
    docker compose --profile dev run --rm app /usr/src/myapp/scripts/run_component_tests.sh

# Run component tests against development server
test-components-dev:
    @echo "Running component tests against development server..."
    @echo "Make sure the development server is running with 'just dev' in another terminal"
    TEST_SERVER_URL=http://app:3000 ./scripts/run_component_tests.sh

# Run integration tests
test-integration:
    @echo "Running integration tests..."
    ./tests/integration/mysql_integration_test.sh

# Run JWT cookie expiration test
test-jwt-cookie-expiry:
    @echo "Running JWT cookie expiration test..."
    ./tests/integration/jwt_cookie_expiry_test.sh

# ===== IDE SETUP =====

# Set up IDE for Rust development
setup-ide:
    @echo "Setting up IDE for Rust development..."
    rustup component add rust-analyzer rust-src

# Generate build data for rust-analyzer
build-data:
    @echo "Generating build data for rust-analyzer..."
    docker compose --profile dev run --rm app cargo check --message-format=json > /dev/null

# Complete IDE setup with build data
setup-ide-complete:
    @echo "Setting up IDE with build data..."
    rustup component add rust-analyzer rust-src
    just build-data

# ===== PRODUCTION =====

# Build production Docker image
build-prod:
    @echo "Building production Docker image..."
    docker compose build app_prod

# Start production environment
prod-up:
    @echo "Starting production environment..."
    docker compose --profile prod up -d || { echo "Failed to start production environment"; exit 1; }

# Stop production environment
prod-down:
    @echo "Stopping production environment..."
    docker compose --profile prod down

# View production logs
prod-logs:
    @echo "Viewing production logs..."
    docker compose --profile prod logs -f

# Run migrations in production environment
prod-migrate:
    @echo "Running migrations in production environment..."
    docker compose --profile prod run --rm app_prod /usr/local/bin/migrate || { echo "Production migration failed"; exit 1; }

# Generate self-signed SSL certificates
generate-ssl:
    @echo "Generating self-signed SSL certificates..."
    ./scripts/generate_ssl_cert.sh
