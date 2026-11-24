#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-current-qualification/materialize-the-frozen-minimal-reference-installation.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
provider_source=tools/benchmarks/fixtures/vue-benchmarks-current-typecheck-provider
provider_prefix="$RUNNER_TEMP/vue-benchmarks-current-provider"
mkdir "$provider_prefix"
cp "$provider_source/package.json" "$provider_source/package-lock.json" "$provider_prefix/"
npm ci --prefix "$provider_prefix" --ignore-scripts --no-audit --no-fund
cmp "$provider_source/package-lock.json" "$provider_prefix/package-lock.json"
sha256sum "$provider_prefix/package.json" "$provider_prefix/package-lock.json"
