# docker build -t rust-operator:1a .
FROM rust:1.52-buster as build

WORKDIR /app

COPY src src
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --release

FROM debian:buster-slim

WORKDIR /app

RUN apt-get update \
 && apt-get install -y libssl-dev \
 && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/rust-operator /app
COPY log4rs.yml .

CMD ["./rust-operator"]
