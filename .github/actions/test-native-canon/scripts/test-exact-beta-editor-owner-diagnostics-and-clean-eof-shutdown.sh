#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-exact-beta-editor-owner-diagnostics-and-clean-eof-shutdown.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
provider="$RUNNER_TEMP/native-beta-10741"
capture="$VIZE_NATIVE_BETA_CAPTURE_DIR"
mkdir -p "$provider" "$capture"
printf '%s\n' '{"private":true,"devDependencies":{"typescript":"7.1.0-beta"}}' >"$provider/package.json"
npm install --prefix "$provider" --ignore-scripts --no-audit --no-fund
cp "$provider/package.json" "$capture/provider-package.json"
cp "$provider/package-lock.json" "$capture/provider-package-lock.json"
if ! command -v strace >/dev/null; then sudo apt-get update && sudo apt-get install -y strace; fi
cargo test --locked --profile ci -p vize_canon --lib --no-run --message-format=json >"$capture/build.jsonl"
node tests/tooling/support/native-beta-shutdown/source.mjs
