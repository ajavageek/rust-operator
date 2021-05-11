# docker build -t rust-operator:1b .
FROM ekidd/rust-musl-builder:1.51.0 as build

WORKDIR /app

COPY src src
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build --release

FROM alpine

WORKDIR /app

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/rust-operator /app
COPY log4rs.yml .

CMD ["./rust-operator"]
