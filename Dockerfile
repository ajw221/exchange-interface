FROM clux/muslrust:stable as builder

COPY .env .
COPY build.rs .
COPY Cargo.lock .
COPY Cargo.toml .
COPY src ./src
COPY objects ./objects

RUN set -x && cargo build --target x86_64-unknown-linux-musl --release

CMD ["./target/x86_64-unknown-linux-musl/release/exchange_interface"]
