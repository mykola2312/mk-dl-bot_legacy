# Rust build
FROM rust:1-alpine3.19

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/mk-dl-bot

# Rust
FROM alpine:3.19
RUN apk add --no-cache libgcc

COPY --from=0 /app/target/release/mk-dl-bot .

ENTRYPOINT ["/mk-dl-bot"]