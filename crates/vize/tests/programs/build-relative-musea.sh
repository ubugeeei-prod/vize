#!/bin/sh
# DO NOT EDIT. Generated from tools/config/pkl/scripts/crates/vize/tests/programs/build-relative-musea.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [ "$1" != "build" ]; then
  exit 9
fi
if [ "${VIZE_MUSEA_STATIC_BUILD:-}" != "1" ]; then
  exit 1
fi
printf 'vite-plus stdout sentinel\n'
mkdir -p dist/__musea__
printf '<!doctype html><title>Musea</title>\n' > dist/__musea__/index.html
