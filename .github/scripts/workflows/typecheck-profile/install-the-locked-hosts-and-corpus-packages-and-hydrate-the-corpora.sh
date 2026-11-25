#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-profile/install-the-locked-hosts-and-corpus-packages-and-hydrate-the-corpora.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund
npm ci --prefix tools/benchmarks/typecheck-corpora --ignore-scripts --no-audit --no-fund
read -ra submodules <<< "$(node tools/benchmarks/scripts/typecheck-profile.ts --print-submodules --corpora "$CORPORA")"
git submodule update --init --depth 1 -- "${submodules[@]}"
