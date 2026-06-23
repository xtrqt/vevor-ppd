# Bonjour And macOS/iOS Printer Support

## Summary

Make the standalone Rust printer app advertise itself as an IPP printer over Bonjour/mDNS and expose enough IPP attributes for macOS and iOS to discover and probe it.

## Requirements

- Bonjour advertisement runs inside the Rust app, not as a separate host Avahi service.
- Primary discovery target is macOS and iOS.
- Advertise `_ipp._tcp.local.` with `rp=ipp/print` on port `631`.
- Keep `/dev/usb/lp1` as the assumed stable NAS printer device for now.
- Keep Bonjour configurable through environment variables.
- Preserve existing Docker and GitHub Actions build flow.

## Initial Scope

- Add a Rust mDNS/DNS-SD advertiser.
- Add config for service name, host address, admin URL, UUID, and enable flag.
- Add macOS/iOS-oriented Bonjour TXT records.
- Improve IPP printer attributes with correct value types for common discovery probes.

## Out Of Scope For This Pass

- Full AirPrint certification.
- URF parsing.
- PDF rendering.
- Full IPP job lifecycle persistence.
- Replacing the placeholder Vevor command generator.

## Testing Plan

- Run `cargo fmt --check`.
- Run `cargo test`.
- Run `cargo build --release`.
- Build the Docker image.
- Smoke-test `GET /health`.
- Smoke-test a minimal IPP `Get-Printer-Attributes` request.
- On NAS, verify `dns-sd -B _ipp._tcp local` from macOS sees the service after deploying the image.
- On macOS/iOS, verify the printer appears in the add/print UI.
