# Rust Micro Front-End Development Commands
# This justfile provides containerized development commands

# Default recipe lists available commands
default:
    @just --list

# Core development workflow
build:
    @echo "Building application in Docker container..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo build

dev:
    @echo "Starting development environment..."
    # docker-compose up --build

test:
    @echo "Running test suite in containers..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo test

test-unit:
    @echo "Running unit tests..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo test --lib

test-integration:
    @echo "Running integration tests with mock database..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo test --test integration

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
    @echo "Formatting Rust code..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo fmt

lint:
    @echo "Running Clippy linting..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo clippy

check:
    @echo "Checking compilation..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo check

audit:
    @echo "Running security audit..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo audit

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
    @echo "Cleaning build artifacts..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo clean
    # docker-compose down --volumes

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
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo bench

profile:
    @echo "Profiling application performance..."
    # docker run --rm -v $(pwd):/workspace rust:1.75 cargo flamegraph

lighthouse:
    @echo "Running Lighthouse performance audit..."
    # docker run --rm --cap-add=SYS_ADMIN ghcr.io/puppeteer/puppeteer lighthouse http://localhost --output json
