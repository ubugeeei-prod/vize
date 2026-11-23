#!/usr/bin/env bash
set -euo pipefail
[[ "$VIZE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
npm install --prefix "$RUNNER_TEMP/admission-provider" --ignore-scripts --no-package-lock --no-audit --no-fund \
  "vize@$VIZE_VERSION" "@vizejs/native@$VIZE_VERSION" vue@3.5.44 vue-tsc@3.3.13 \
  typescript@6.0.3 typescript-native@npm:typescript@7.1.0-beta
