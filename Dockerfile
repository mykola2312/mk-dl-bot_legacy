# Rust build
FROM rust:1-alpine3.19

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY src/ /app/src

RUN cargo build --release
RUN strip target/release/mk-dl-bot

# Run
FROM alpine:3.19
RUN apk add --no-cache libgcc
RUN apk add --no-cache ffmpeg python3 py3-pip
RUN pip install --break-system-packages yt-dlp

COPY --from=0 /app/target/release/mk-dl-bot .

ENTRYPOINT ["/mk-dl-bot"]