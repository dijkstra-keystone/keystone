FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY backend ./backend

# Build release binary
RUN cargo build --release --package keystone-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/keystone-api /usr/local/bin/
COPY --from=builder /app/backend/migrations /migrations

ENV PORT=8080
EXPOSE 8080

CMD ["keystone-api"]
