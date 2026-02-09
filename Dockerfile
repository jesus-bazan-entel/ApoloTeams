# =============================================================================
# Stage 1: Build Frontend (React/Vite)
# =============================================================================
FROM node:20-alpine AS frontend-builder

WORKDIR /build/frontend

# Install dependencies first (cached layer)
COPY frontend-react/package.json frontend-react/package-lock.json* ./
RUN npm ci --prefer-offline

# Copy frontend source and build
COPY frontend-react/ ./
RUN npm run build

# =============================================================================
# Stage 2: Build Backend (Rust/Actix-web)
# =============================================================================
FROM rust:1.92-bookworm AS backend-builder

WORKDIR /build

# Install system dependencies for SQLx (PostgreSQL client libs)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first (cached dependency layer)
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml

# Create minimal source stubs so cargo can resolve the workspace
# This lets Docker cache the dependency download step
RUN mkdir -p backend/src shared/src frontend/src && \
    echo "fn main() {}" > backend/src/main.rs && \
    echo "" > shared/src/lib.rs && \
    echo "fn main() {}" > frontend/src/main.rs

# Pre-build dependencies (cached unless Cargo.toml/lock changes)
ENV SQLX_OFFLINE=true
RUN cargo build --release -p backend 2>/dev/null || true

# Now copy actual source code
COPY backend/src/ backend/src/
COPY shared/src/ shared/src/
COPY backend/migrations/ backend/migrations/

# Touch source files to invalidate cached stubs, then build
RUN touch backend/src/main.rs shared/src/lib.rs && \
    cargo build --release -p backend

# =============================================================================
# Stage 3: Runtime (minimal Debian image)
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r apolo && useradd -r -g apolo -d /app apolo

WORKDIR /app

# Copy the compiled binary
COPY --from=backend-builder /build/target/release/rust-teams-server ./

# Copy migrations (SQLx runs them on startup)
COPY --from=backend-builder /build/backend/migrations/ ./migrations/

# Copy built frontend into static directory
COPY --from=frontend-builder /build/frontend/dist/ ./static/

# Create required directories with correct ownership
RUN mkdir -p ./data ./uploads && \
    chown -R apolo:apolo /app

USER apolo

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./rust-teams-server"]