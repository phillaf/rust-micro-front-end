# Development stage - used for building during development
FROM rust:1.75 as dev

RUN apt-get update && apt-get install -y tini && rm -rf /var/lib/apt/lists/*

# Create a user with UID 1000 (typical for first Linux user)
RUN groupadd --gid 1000 developer && \
    useradd --uid 1000 --gid 1000 --shell /bin/bash --create-home developer && \
    echo "developer:developer" | chpasswd && \
    usermod -aG sudo developer

# Create directory structure and set permissions
RUN mkdir -p /usr/src/myapp && chown -R developer:developer /usr/src/myapp

# Ensure PATH includes cargo binaries
ENV PATH="/usr/local/cargo/bin:${PATH}"
EXPOSE 80

WORKDIR /usr/src/myapp
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/bin/bash"]

# Build stage - used for compiling production binaries
FROM rust:1.75 as builder

WORKDIR /build
COPY . .

# Install the build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config libssl-dev musl-tools && \
    rm -rf /var/lib/apt/lists/*

# Install the musl target for Alpine compatibility
RUN rustup target add x86_64-unknown-linux-musl

# Build the application in release mode for Alpine (musl)
RUN cargo build --release --target x86_64-unknown-linux-musl

# Production stage - minimal Alpine image with just the compiled binary
FROM alpine:3.19 as production

# Create a non-root user to run the application
RUN addgroup -S app && adduser -S -G app app

# Install minimal runtime dependencies
RUN apk --no-cache add \
    ca-certificates \
    tini

# Copy only the compiled binary from the builder stage (musl target)
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/rust-micro-front-end /usr/local/bin/
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/migrate /usr/local/bin/

# Copy templates and static assets needed for runtime
COPY --chown=app:app templates /usr/src/myapp/templates

WORKDIR /usr/src/myapp
USER app

# Expose the port the application will run on
EXPOSE 80

# Use tini as init system to handle signals properly (Alpine path)
ENTRYPOINT ["/sbin/tini", "--"]

# Run the application
CMD ["rust-micro-front-end"]