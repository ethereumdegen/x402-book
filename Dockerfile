# Frontend build stage
FROM node:20-slim AS frontend-builder

WORKDIR /app/forum-frontend

# Copy frontend package files
COPY forum-frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY forum-frontend/ ./

# Build frontend
RUN npm run build

# Backend build stage
FROM rust:1.88-slim-bookworm AS backend-builder

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
COPY --from=backend-builder /app/target/release/forum-backend /app/

# Copy migrations for runtime migrations
COPY migrations /app/migrations

# Copy the built frontend (dist folder)
COPY --from=frontend-builder /app/forum-frontend/dist /app/forum-frontend/dist

# Expose port
EXPOSE 8080

# Run the application
CMD ["/app/forum-backend"]
