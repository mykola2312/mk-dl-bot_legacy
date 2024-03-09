# Rust build
FROM rust:1-alpine3.19 AS builder

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /usr/src/app
RUN USER=root cargo init
COPY Cargo.toml .
COPY Cargo.lock .
COPY src/ ./src
COPY locales/ ./locales/
COPY migrations/ ./migrations/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    touch src/main.rs && cargo build --release
RUN strip target/release/mk-dl-bot

# Run
FROM alpine:3.19 AS final
# Dependencies
RUN apk add --no-cache libgcc
RUN apk add --no-cache ffmpeg python3 py3-pip
RUN pip install --break-system-packages yt-dlp

WORKDIR /app
COPY migrations/ ./migrations/
COPY locales/ ./locales/
COPY --from=builder /usr/src/app/target/release/mk-dl-bot .

ENTRYPOINT ["/app/mk-dl-bot"]