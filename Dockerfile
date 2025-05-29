# syntax=docker/dockerfile:1.4

FROM rust:1.84-alpine AS builder
RUN apk add --no-cache build-base protobuf-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo fetch
COPY proto ./proto
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --bin rbln-metrics-exporter && \
    cp /app/target/release/rbln-metrics-exporter /usr/local/bin/

FROM alpine:3.18
COPY --from=builder /usr/local/bin/rbln-metrics-exporter /usr/local/bin/rbln-metrics-exporter
ENV RUST_LOG=info
EXPOSE 9090
ENTRYPOINT ["/usr/local/bin/rbln-metrics-exporter"]
