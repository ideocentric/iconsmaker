# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:slim-bookworm AS builder

WORKDIR /build

# Fetch and compile all dependencies before copying source.
# This layer is cached and only invalidated when Cargo.toml or Cargo.lock changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main(){}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Compile the real source. Touch main.rs so cargo sees a newer mtime than the
# cached dummy build and recompiles the binary.
COPY src/ ./src/
RUN touch src/main.rs \
    && cargo build --release

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

COPY --from=builder /build/target/release/iconsmaker /usr/local/bin/iconsmaker

# The squircle mask is generated analytically at runtime — no bundled SVG needed.

# /workspace is the mount point for the project being processed.
WORKDIR /workspace

ENTRYPOINT ["iconsmaker"]

# Default: look for icons.toml in the mounted working directory.
# Override at run time:  docker run ... iconsmaker --config path/to/other.toml
CMD ["--config", "icons.toml"]