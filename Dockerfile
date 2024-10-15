FROM rust:latest

WORKDIR /app
COPY . .
RUN cargo build --release

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
EXPOSE 8000
CMD ["target/release/tmlapis"]
