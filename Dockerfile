FROM rust:1-bookworm AS build

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends avahi-utils ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/vevor-printer-app /usr/local/bin/vevor-printer-app
COPY docker/entrypoint.sh /usr/local/bin/vevor-printer-entrypoint
RUN chmod +x /usr/local/bin/vevor-printer-entrypoint

EXPOSE 631
ENV LISTEN_ADDR=0.0.0.0:631
ENV OUTPUT_DEVICE=/dev/usb/lp0
ENV ENABLE_BONJOUR=false
ENV ENABLE_AVAHI_PUBLISH=false
ENV PRINTER_HOST=localhost

ENTRYPOINT ["/usr/local/bin/vevor-printer-entrypoint"]
