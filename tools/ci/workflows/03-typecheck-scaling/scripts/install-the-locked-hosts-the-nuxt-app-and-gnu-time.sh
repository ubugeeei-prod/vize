#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-scaling/scripts/install-the-locked-hosts-the-nuxt-app-and-gnu-time.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund
if [[ -n "$NUXT_SIZES" ]]; then npm ci --prefix tools/benchmarks/nuxt-app --ignore-scripts --no-audit --no-fund; fi
if ! /usr/bin/time --version 2>&1 | grep -qi 'GNU time'; then
  sudo apt-get update -qq
  sudo apt-get install -y --no-install-recommends time
fi
/usr/bin/time --version
