# syntax=docker/dockerfile:1.7

# ============================================================================
# Build Stage
# ============================================================================
FROM rust:1.81-bookworm AS build

WORKDIR /src

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY examples ./examples

# Build release binary with optimizations
RUN cargo build --release && \
    strip /src/target/release/sap-adapter

# ============================================================================
# Runtime Stage (Distroless)
# ============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot

USER nonroot:nonroot
WORKDIR /app

# Copy binary from build stage
COPY --from=build /src/target/release/sap-adapter /app/sap-adapter

# Copy examples
COPY examples /app/examples

# Expose (no ports needed for client)
# Adapter is a client, not a server

# Run the adapter
ENTRYPOINT ["/app/sap-adapter"]
CMD ["--dry-run"]
