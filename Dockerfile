FROM rust:1.58 AS builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/carbon-alert /usr/local/bin
ENV RUST_LOG carbon_alert=info
ENTRYPOINT ["/usr/local/bin/carbon-alert"]
