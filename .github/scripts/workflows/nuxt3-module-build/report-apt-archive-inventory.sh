#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/nuxt3-module-build/report-apt-archive-inventory.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
case "${1:-after-update}" in
  after-update) echo 'Nuxt3 archive inventory after package-index update' ;;
  before-install) echo 'Nuxt3 archive inventory before the original Chromium installation' ;;
  *) exit 2 ;;
esac
find "$(dirname "$0")" -maxdepth 1 -type f -name '*.deb' -print0 | sort -z | xargs -0 -r sha256sum
