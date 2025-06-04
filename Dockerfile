FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install dependencies for vendored OpenSSL build
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Create a non-root user
RUN useradd -m appuser

# Copy only the compiled binary from the builder stage
COPY --from=builder /app/target/release/tmlapis /usr/local/bin/tmlapis
COPY --from=builder /app/img /app/img

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

# Use non-root user
USER appuser
WORKDIR /app
EXPOSE 8000
CMD ["tmlapis"]
