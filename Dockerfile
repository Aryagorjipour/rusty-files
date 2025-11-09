FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build the server binary
RUN cargo build --release --bin filesearch-server

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/filesearch-server /usr/local/bin/

# Copy configuration files
COPY --from=builder /app/config /etc/filesearch/

# Create data directory
RUN mkdir -p /data

EXPOSE 8080

ENTRYPOINT ["filesearch-server"]
CMD ["--config", "/etc/filesearch/production.toml"]
