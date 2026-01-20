FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock Rocket.toml ./
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

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
