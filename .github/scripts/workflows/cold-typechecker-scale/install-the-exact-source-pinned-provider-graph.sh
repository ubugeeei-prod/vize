#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/install-the-exact-source-pinned-provider-graph.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$(node --version)" == v24.14.0 ]]
[[ "$(npm --version)" == 11.9.0 ]]
mkdir "$RUNNER_TEMP/provider"
cp tests/_fixtures/tooling/owned-read-buffer-provider/package.json.txt "$RUNNER_TEMP/provider/package.json"
cp tests/_fixtures/tooling/owned-read-buffer-provider/package-lock.json.txt "$RUNNER_TEMP/provider/package-lock.json"
cd "$RUNNER_TEMP/provider"
printf '%s\n' 'bb62ff5452cbfd7866d232c6e1a78d1d7b6d82ec5f6cbf67e08aa6deff9c324e  package.json' '9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6  package-lock.json' | sha256sum --check -
npm ci --ignore-scripts --no-audit --no-fund
cp package.json package-lock.json "$RUNNER_TEMP/cold-paired/"
