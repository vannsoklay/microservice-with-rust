# Build stage
FROM rust:1.89 as builder

WORKDIR /app
COPY . .

# Build only this service binary
RUN cargo build --release -p gateway

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
&& rm -rf /var/lib/apt/lists/*

# Copy binary from build stage
COPY --from=builder /app/target/release/gateway /app/gateway

# Copy .env if needed
COPY .env /app/.env

EXPOSE 8000

CMD ["./gateway"]
