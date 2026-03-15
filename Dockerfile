FROM rust:1.85-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libsqlite3-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/feedhook /usr/local/bin/
EXPOSE 8080
VOLUME /data
ENV DATABASE_URL=sqlite:///data/feedhook.db
CMD ["feedhook"]
