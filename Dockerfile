FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runner
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/CosmicComicsRustServer .
EXPOSE 4696
CMD ["./CosmicComicsRustServer"]
