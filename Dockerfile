# Development stage - used for building during development
FROM rust:1.75 as dev

RUN apt-get update && apt-get install -y tini && rm -rf /var/lib/apt/lists/*

EXPOSE 80

WORKDIR /app
ENTRYPOINT ["/usr/bin/tini", "--"]

# Build stage - used for compiling production binaries
FROM rust:1.75 as builder

WORKDIR /build
COPY . .

# Install the build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build the application in release mode
RUN cargo build --release

# Production stage - minimal image with just the compiled binary
FROM debian:bookworm-slim as production

# Create a non-root user to run the application
RUN groupadd -r app && useradd -r -g app app

# Install runtime dependencies and security updates
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    tini \
    libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy only the compiled binary from the builder stage
COPY --from=builder /build/target/release/rust-micro-front-end /usr/local/bin/
COPY --from=builder /build/target/release/migrate /usr/local/bin/

# Copy templates and static assets needed for runtime
COPY --chown=app:app templates /app/templates

WORKDIR /app
USER app

# Expose the port the application will run on
EXPOSE 80

# Use tini as init system to handle signals properly
ENTRYPOINT ["/usr/bin/tini", "--"]

# Run the application
CMD ["rust-micro-front-end"]