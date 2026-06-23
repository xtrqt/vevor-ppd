FROM rust:1-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/vevor-printer-app /usr/local/bin/vevor-printer-app

EXPOSE 631
ENV LISTEN_ADDR=0.0.0.0:631
ENV OUTPUT_DEVICE=/dev/usb/lp0

ENTRYPOINT ["/usr/local/bin/vevor-printer-app"]
