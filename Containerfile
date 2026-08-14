# ==============================================================================
# ChiPanel Multi-Stage Containerfile
# Target Platform: linux/amd64, Debian runtime
# Final Image Size Target: < 50MB
# ==============================================================================

# Stage 1: Build SvelteKit Static Frontend
FROM node:22-slim AS frontend-builder
WORKDIR /app/frontend

COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci || (npm install --package-lock-only && npm ci)

COPY frontend/ ./
RUN npm run build

# Stage 2: Build Rust Axum Release Binary
# Pin to bookworm so the binary links against the same glibc (2.36) as the
# bookworm-slim runtime stage below — an unpinned `rust:slim` now tracks trixie
# (glibc 2.39) and produced binaries that fail at startup with GLIBC_2.39 not found.
FROM rust:bookworm AS rust-builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend

# Dependency layer caching
COPY backend/Cargo.toml backend/Cargo.lock* ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy real source code and touch main.rs to trigger rebuild
COPY backend/src ./src
COPY --from=frontend-builder /app/frontend/build /app/frontend/build
RUN touch src/main.rs
RUN cargo build --release --bin chipanel && \
    strip /app/backend/target/release/chipanel

# Stage 3: Minimal Debian Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /app/backend/target/release/chipanel /app/chipanel
COPY --from=frontend-builder /app/frontend/build /app/frontend/build

ENV PORT=3000
ENV RUST_LOG=info

EXPOSE 3000
USER 1000:1000
ENTRYPOINT ["/app/chipanel"]
