#!/bin/sh
if [ "$1" != "build" ]; then
  exit 9
fi
if [ "${VIZE_MUSEA_STATIC_BUILD:-}" != "1" ]; then
  exit 1
fi
printf 'vite-plus stdout sentinel\n'
mkdir -p dist/__musea__
printf '<!doctype html><title>Musea</title>\n' > dist/__musea__/index.html
