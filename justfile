# Rust Micro Front-End Development Commands
# This justfile provides containerized development commands

# Get current user and group IDs to avoid permission issues
export UID := `id -u`
export GID := `id -g`

# Default recipe lists available commands
default:
    @just --list

bash:
    docker compose run -it --rm app bash

build:
    docker compose run --rm app cargo build

dev:
    docker compose run --rm --service-ports app cargo run --bin rust-micro-front-end

test:
    @echo "Running test suite in containers..."
    docker compose run --rm app cargo test -- --test-threads=1

test-parallel:
    @echo "Running test suite in containers (parallel)..."
    docker compose run --rm app cargo test

test-unit:
    @echo "Running unit tests..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo test --lib

test-integration:
    @echo "Running integration tests..."
    ./tests/integration/mysql_integration_test.sh

test-jwt:
    @echo "Running JWT authentication tests..."
    ./tests/integration/jwt_test.sh

# IDE tools

# IDE support commands (runs once to configure local development)
setup-ide:
    rustup component add rust-analyzer rust-src

# IDE support - generate build data for rust-analyzer
build-data:
    docker compose run --rm app cargo check --message-format=json > /dev/null

# Complete IDE setup including build data
setup-ide-complete:
    rustup component add rust-analyzer rust-src
    just build-data

# Database operations
db-up:
    @echo "Starting MySQL database..."
    docker compose up -d mysql
    @echo "Waiting for MySQL to be ready..."
    docker compose exec mysql bash -c 'until mysqladmin ping -h localhost --silent; do sleep 1; done'
    @echo "MySQL is ready!"

migrate:
    @echo "Running database migrations..."
    docker compose run --rm app cargo run --bin migrate

migrate-reset:
    @echo "Resetting database and running all migrations..."
    # docker-compose exec app sqlx database reset

seed:
    @echo "Seeding database with test data..."
    # docker-compose exec app cargo run --bin seed

db-shell:
    @echo "Accessing database shell..."
    docker compose exec mysql bash -c 'mysql -u "$MYSQL_USER" -p"$MYSQL_PASSWORD" "$MYSQL_DATABASE"'

# Code quality
format:
    docker compose run --rm app cargo fmt

lint:
    docker compose run --rm app cargo clippy

check:
    docker compose run --rm app cargo check

audit:
    # docker compose run --rm app cargo audit

# Development utilities
logs:
    @echo "Viewing application logs..."
    # docker-compose logs -f app

logs-db:
    @echo "Viewing database logs..."
    # docker-compose logs -f mysql

logs-nginx:
    @echo "Viewing nginx logs..."
    # docker-compose logs -f nginx

clean:
    docker compose run --rm app cargo clean

# Fix permission issues with target directory (run this if you get permission errors)
fix-permissions:
    sudo chown -R ${UID}:${GID} target/ || true

# Initialize cargo and rustup cache volumes with correct permissions
init-cargo-cache:
    @echo "Initializing cargo and rustup caches with correct permissions..."
    docker compose run --rm --user root app chown -R ${UID}:${GID} /usr/local/cargo || true
    docker compose run --rm --user root app chown -R ${UID}:${GID} /usr/local/rustup || true
    @echo "Cargo and rustup caches initialized"

# Nuclear clean - removes target directory entirely and recreates with correct permissions
clean-nuclear:
    sudo rm -rf target/
    just init-cargo-cache
    docker compose run --rm app cargo check > /dev/null

reset:
    @echo "Nuclear reset - rebuilding everything..."
    # docker-compose down --volumes --rmi all
    # docker system prune -f

# JWT testing utilities
jwt-generate:
    @echo "Generating test JWT tokens..."
    # docker run --rm -v $(pwd)/scripts:/scripts node:18 node /scripts/generate-jwt.js

jwt-validate:
    @echo "Validating JWT token format..."
    # docker run --rm -v $(pwd)/scripts:/scripts node:18 node /scripts/validate-jwt.js

# Performance and monitoring
benchmark:
    @echo "Running performance benchmarks..."
    # docker run --rm -v $(pwd):/workspace --user ${UID}:${GID} rust:1.75 cargo bench

profile:
    @echo "Profiling application performance..."
    # docker run --rm -v $(pwd):/workspace --user ${UID}:${GID} rust:1.75 cargo flamegraph

lighthouse:
    @echo "Running Lighthouse performance audit..."
    # docker run --rm --cap-add=SYS_ADMIN ghcr.io/puppeteer/puppeteer lighthouse http://localhost --output json

test-module module_name:
    @echo "Running specific test module: {{module_name}}..."
    docker compose run --rm app cargo test {{module_name}} -- --test-threads=1

# Micro Front-End Component Testing
test-components:
    @echo "Running component isolation tests..."
    docker compose run --rm app /app/scripts/run_component_tests.sh

test-components-dev:
    @echo "Running component tests with development server..."
    @echo "Make sure the development server is running with 'just dev' in another terminal"
    TEST_SERVER_URL=http://app:3000 ./scripts/run_component_tests.sh

# Production commands
generate-ssl:
    @echo "Generating self-signed SSL certificates for development..."
    ./scripts/generate_ssl_cert.sh

# Backup and recovery commands
backup-database:
    @echo "Backing up database..."
    docker compose run --rm app /scripts/backup/db_full_backup.sh

backup-config:
    @echo "Backing up configuration..."
    docker compose run --rm app /scripts/backup/config_backup.sh

backup-images:
    @echo "Backing up container images..."
    ./scripts/backup/image_backup.sh

backup-all: backup-database backup-config backup-images
    @echo "All backups completed successfully"

recover-database BACKUP_FILE:
    @echo "Recovering database from {{BACKUP_FILE}}..."
    docker compose run --rm app /scripts/recovery/db_full_recovery.sh {{BACKUP_FILE}}

recover-database-point-in-time BACKUP_FILE TIMESTAMP:
    @echo "Recovering database to {{TIMESTAMP}}..."
    docker compose run --rm app /scripts/recovery/db_point_in_time_recovery.sh {{BACKUP_FILE}} "{{TIMESTAMP}}"

recover-config CONFIG_BACKUP:
    @echo "Recovering configuration from {{CONFIG_BACKUP}}..."
    docker compose run --rm app /scripts/recovery/config_recovery.sh {{CONFIG_BACKUP}}

recover-image IMAGE_BACKUP:
    @echo "Recovering image from {{IMAGE_BACKUP}}..."
    ./scripts/recovery/image_recovery.sh {{IMAGE_BACKUP}}

validate-backups:
    @echo "Validating backups..."
    docker compose run --rm app /scripts/validation/backup_validation.sh

# Composability testing
test-composability:
    @echo "Running composability validation tests..."
    docker compose run --rm app /tests/composability/validate_composability.sh

build-prod:
    @echo "Building production Docker image..."
    docker compose build app_prod

prod-up:
    @echo "Starting production environment..."
    docker compose --profile prod up -d

prod-down:
    @echo "Stopping production environment..."
    docker compose --profile prod down

prod-logs:
    @echo "Viewing production logs..."
    docker compose --profile prod logs -f

prod-migrate:
    @echo "Running database migrations in production environment..."
    docker compose --profile prod run --rm app_prod /usr/local/bin/migrate
