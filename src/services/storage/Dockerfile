# Build stage
FROM rust:1.89 as builder

WORKDIR /app
COPY . .

# Build only this service binary
RUN cargo build --release -p storage

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
&& rm -rf /var/lib/apt/lists/*

# Copy binary from build stage
COPY --from=builder /app/target/release/storage /app/storage

# Copy .env if needed
COPY uploads /app/uploads
COPY .env /app/.env

EXPOSE 8086

CMD ["./storage"]
