# Multi-Stage Dockerfile for CAP Agent & Verifier API
FROM rust:1.75-slim-bullseye AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests
RUN cargo build --release --bin cap-agent && cargo build --release --bin cap-verifier-api
RUN strip /app/target/release/cap-agent && strip /app/target/release/cap-verifier-api

FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 curl && rm -rf /var/lib/apt/lists/*
RUN useradd -m -u 1000 -s /bin/bash capuser
WORKDIR /app
RUN mkdir -p /app/build /app/keys /app/config && chown -R capuser:capuser /app
COPY --from=builder --chown=capuser:capuser /app/target/release/cap-agent /usr/local/bin/
COPY --from=builder --chown=capuser:capuser /app/target/release/cap-verifier-api /usr/local/bin/
USER capuser
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s CMD curl -f http://localhost:8080/healthz || exit 1
CMD ["cap-verifier-api", "--bind", "0.0.0.0:8080"]
