# Stage 1: Build the application
# We use cargo-chef to cache dependencies
FROM rust:1.78 as chef
WORKDIR /app
RUN cargo install cargo-chef

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin PersonaForge

# Stage 2: Create the final, minimal image
FROM debian:slim-bullseye as runtime
WORKDIR /app
# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/PersonaForge /usr/local/bin/
# Copy .env file for configuration, ensure it's present during runtime
# COPY .env .
# Needed for reqwest to find SSL certificates
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Run the binary
CMD ["/usr/local/bin/PersonaForge"]
