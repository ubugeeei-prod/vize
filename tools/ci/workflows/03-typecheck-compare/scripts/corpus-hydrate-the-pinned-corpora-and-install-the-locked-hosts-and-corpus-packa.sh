#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/corpus-hydrate-the-pinned-corpora-and-install-the-locked-hosts-and-corpus-packa.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
git submodule update --init --depth 1 -- tests/_fixtures/_git/misskey tests/_fixtures/_git/n8n tests/_fixtures/_git/nuxt-ui tests/_fixtures/_git/reka-ui
npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund
npm ci --prefix tools/benchmarks/typecheck-hosts-tsrs-0.23 --ignore-scripts --no-audit --no-fund
npm ci --prefix tools/benchmarks/typecheck-corpora --ignore-scripts --no-audit --no-fund
if [[ "$OWNED_PAIR" == true ]]; then
  npm ci --prefix tools/benchmarks/nuxt-app --no-audit --no-fund
fi
# The vue-tsc reference and its TypeScript 6, as the workspace lockfile pins them.
vp install --frozen-lockfile --prefer-offline --filter './tools/benchmarks/scripts'
if ! /usr/bin/time --version 2>&1 | grep -qi 'GNU time'; then
  sudo apt-get update -qq
  sudo apt-get install -y --no-install-recommends time
fi
/usr/bin/time --version
