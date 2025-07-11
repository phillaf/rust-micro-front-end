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
    docker compose run --rm --service-ports app cargo run

test:
    @echo "Running test suite in containers..."
    docker compose run --rm app cargo test

test-unit:
    @echo "Running unit tests..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo test --lib

test-integration:
    @echo "Running integration tests with mock database..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo test --test integration

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
migrate:
    @echo "Running database migrations..."
    # docker-compose exec app sqlx migrate run

migrate-reset:
    @echo "Resetting database and running all migrations..."
    # docker-compose exec app sqlx database reset

seed:
    @echo "Seeding database with test data..."
    # docker-compose exec app cargo run --bin seed

db-shell:
    @echo "Accessing database shell..."
    # docker-compose exec mysql mysql -u app_user -p micro_frontend

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
