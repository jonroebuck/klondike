FROM rust:1.87 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin klondike-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/klondike-server /usr/local/bin/klondike-server

VOLUME /data
ENV DATABASE_URL=sqlite:///data/klondike.db?mode=rwc
ENV PORT=3000
EXPOSE 3000

CMD ["klondike-server"]
