# syntax=docker/dockerfile:1.6
FROM rust:1.90-bookworm AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 -g nogroup app

WORKDIR /app

COPY --from=builder /app/target/release/blog /app/blog
COPY --from=builder /app/build /app/build

USER app

EXPOSE 3000

ENTRYPOINT ["/app/blog"]
