#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/06-native-smoke/scripts/fresh-packed-nuxt-init-check-and-lsp-smoke.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
read -ra js <<< "$RELEASE_JS_PACKAGES"
node tools/commands/release/npm/smoke-release-install.ts --prepare-manifests --runtime-checks --bun-runtime-checks \
  npm/native npm/native/npm/* "${js[@]}"
