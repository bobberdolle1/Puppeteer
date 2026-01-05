# Stage 1: Build the application
FROM rust:1.83 AS chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin PersonaForge

# Stage 2: Create the final, minimal image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary (static files are embedded via rust_embed)
COPY --from=builder /app/target/release/PersonaForge /usr/local/bin/

# Copy migrations for database setup
COPY migrations/ /app/migrations/

# SSL certificates for reqwest
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

# Create data directory for database
RUN mkdir -p /app/data

# Set working directory
WORKDIR /app

# Expose webapp port
EXPOSE 8080

# Run the binary
CMD ["/usr/local/bin/PersonaForge"]
