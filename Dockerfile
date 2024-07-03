FROM rust:bookworm as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && touch src/lib.rs
RUN cargo check
COPY src ./src
RUN cargo build --features="redis" --release

FROM debian:bookworm-slim
RUN apt-get update && apt install -y openssl ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/konnektoren-api /usr/local/bin/konnektoren-api
CMD ["konnektoren-api"]
