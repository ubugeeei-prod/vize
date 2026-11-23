#!/usr/bin/env bash
set -euo pipefail
npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund
npm ci --prefix tools/benchmarks/typecheck-corpora --ignore-scripts --no-audit --no-fund
read -ra submodules <<< "$(node tools/benchmarks/scripts/typecheck-profile.ts --print-submodules --corpora "$CORPORA")"
git submodule update --init --depth 1 -- "${submodules[@]}"
