# Backend-only Dockerfile for Railway deployment
# (Frontend deploys separately to Vercel)

FROM rust:1.88-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY forum-backend ./forum-backend

# Build the application
RUN cargo build --release -p forum-backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/forum-backend /app/

# Copy migrations for runtime migrations
COPY migrations /app/migrations

# Expose port (Railway sets PORT env var)
EXPOSE 8080

# Run the application
CMD ["/app/forum-backend"]
