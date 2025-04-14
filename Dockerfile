FROM rust:latest
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY . .
RUN cargo build --release
EXPOSE 4696
CMD ["./target/release/CosmicComicsRustServer"]