#!/bin/sh
set -eu

if [ "${ENABLE_AVAHI_PUBLISH:-false}" = "true" ]; then
  service_name="${BONJOUR_SERVICE_NAME:-${PRINTER_NAME:-Vevor Label Printer 300}}"
  port="${BONJOUR_PORT:-631}"
  host="${PRINTER_HOST:-localhost}"
  uuid="${BONJOUR_UUID:-8a8a9a2d-43dc-4c7f-8fd3-0e4f03000001}"

  avahi-publish-service \
    --subtype=_universal._sub._ipp._tcp \
    "$service_name" \
    _ipp._tcp \
    "$port" \
    txtvers=1 \
    qtotal=1 \
    rp=ipp/print \
    "ty=${PRINTER_NAME:-Vevor Label Printer 300}" \
    'product=(Vevor Label Printer 300)' \
    pdl=image/pwg-raster \
    kind=document,label \
    PaperMax=legal-A4 \
    "adminurl=http://${host}:${port}/" \
    "UUID=${uuid}" \
    Color=F \
    Duplex=F \
    'note=Vevor Label Printer' \
    priority=50 &
fi

exec /usr/local/bin/vevor-printer-app "$@"
