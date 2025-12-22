# Multi-stage build for minimal image size
FROM rust:1.77-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/kythia-relay /app/kythia-relay

# Create non-root user
RUN addgroup -g 1000 kythia && \
    adduser -D -u 1000 -G kythia kythia && \
    chown -R kythia:kythia /app

USER kythia

EXPOSE 8080 8081

ENTRYPOINT ["/app/kythia-relay"]
