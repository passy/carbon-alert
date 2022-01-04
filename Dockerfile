FROM rust as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin carbon-alert

FROM debian:stable-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/carbon-alert /usr/local/bin
ENTRYPOINT ["/usr/local/bin/carbon-alert"]