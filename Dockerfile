# Frigate Configuration Tool - Phase 8
# Multi-stage build for optimized production image

# Stage 1: Build frontend
FROM node:18-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy frontend package files
COPY src-ui/package*.json ./

# Install frontend dependencies (including dev dependencies needed for build)
RUN npm ci

# Copy frontend source
COPY src-ui/ ./

# Build frontend for production
RUN npm run build

# Stage 2: Build Rust backend
FROM rust:1.82-slim AS backend-builder

WORKDIR /app

# Install system dependencies needed for Tauri
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace Cargo files (required for workspace)
COPY Cargo.toml Cargo.lock ./

# Copy Rust source code
COPY src-tauri/ ./src-tauri/

# Copy built frontend to expected location
COPY --from=frontend-builder /app/frontend/dist ./src-ui/dist/

# Build release binary from workspace root
RUN cargo build --release --manifest-path=src-tauri/Cargo.toml

# Stage 3: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-37 \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder (built from workspace root)
COPY --from=backend-builder /app/target/release/frigate-config-tool /app/
COPY --from=frontend-builder /app/frontend/dist /app/web/

# Create data directory
RUN mkdir -p /app/data

# Set environment variables
ENV RUST_LOG=info
ENV FRIGATE_CONFIG_DATA_DIR=/app/data
ENV FRIGATE_HTTP_MODE=true
ENV PORT=1420

# Expose port for HTTP server
EXPOSE 1420

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:1420/ || exit 1

# Run the application
CMD ["/app/frigate-config-tool"]
