#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/replay-hydrate-the-corpora-and-install-the-locked-hosts-and-corpus-packages.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
chmod +x replay-bin/*/*/vize
modules=()
for corpus in ${CORPORA//,/ }; do modules+=("tests/_fixtures/_git/$corpus"); done
git submodule update --init --depth 1 -- "${modules[@]}"
npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund
npm ci --prefix tools/benchmarks/typecheck-corpora --ignore-scripts --no-audit --no-fund
packages=(time)
[[ "$PROFILE" == true ]] && packages+=(strace linux-tools-generic)
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends "${packages[@]}" > /dev/null
