#!/bin/sh
if [ "$1" != "build" ]; then
  exit 9
fi
if [ "${VIZE_MUSEA_STATIC_BUILD:-}" != "1" ]; then
  printf 'Could not resolve entry module "index.html"\n' >&2
  exit 1
fi
printf 'vite stdout sentinel\n'
printf 'vite stderr sentinel\n' >&2
if [ "${VIZE_MUSEA_TEST_FAIL:-}" = "1" ]; then
  exit 7
fi
mkdir -p dist/__musea__
printf '<!doctype html><title>Musea</title>\n' > dist/__musea__/index.html
