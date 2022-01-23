FROM rust AS builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM debian:stable-slim AS runtime
COPY --from=builder /usr/local/cargo/bin/carbon-alert /usr/local/bin
ENTRYPOINT ["/usr/local/bin/carbon-alert"]
