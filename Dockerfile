FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/planka-mcp /usr/local/bin/planka-mcp

ENTRYPOINT ["planka-mcp"]
