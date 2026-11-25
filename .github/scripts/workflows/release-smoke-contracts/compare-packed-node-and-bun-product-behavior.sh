#!/usr/bin/env bash
node tools/commands/release/npm/smoke-release-install.ts \
  --prepare-manifests \
  --runtime-checks \
  --bun-runtime-checks npm/native npm/native/npm/* npm/cli npm/builder/vite npm/linter/oxlint
