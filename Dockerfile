# Build Stage
FROM rust:1.85-bullseye AS builder

WORKDIR /app

# Install native dependencies required by Slint and SQLite
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    pkg-config \
    libfontconfig1-dev \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libxkbcommon-x11-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and source
COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ ./src/
COPY ui/ ./ui/
COPY tests/ ./tests/

# Run tests and build release
RUN cargo test --release
RUN cargo build --release

# Runtime Stage
FROM debian:bullseye-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libfontconfig1 \
    libx11-6 \
    libxcursor1 \
    libxrandr2 \
    libxi6 \
    libgl1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fazpos /app/fazpos
COPY --from=builder /app/target/release/license_cli /app/license_cli

ENV SLINT_BACKEND=winit

CMD ["/app/fazpos"]
