FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim AS runner
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/CosmicComicsRustServer .
EXPOSE 4696
CMD ["./CosmicComicsRustServer"]