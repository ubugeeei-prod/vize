#!/usr/bin/env bash
set -o pipefail
rustc -Vv > "$RUNNER_TEMP/n8n-whole-corpus-workflow/rustc.txt"
cargo -V > "$RUNNER_TEMP/n8n-whole-corpus-workflow/cargo.txt"
vp node --version > "$RUNNER_TEMP/n8n-whole-corpus-workflow/node.txt"
sha256sum Cargo.lock pnpm-lock.yaml tools/benchmarks/scripts/package.json > "$RUNNER_TEMP/n8n-whole-corpus-workflow/dependency-inputs.sha256"
cargo build --profile ci -p vize 2>&1 | tee "$RUNNER_TEMP/n8n-whole-corpus-workflow/build.log"
vp node tests/differential/build-receipt.ts
cp target/ci/vize.differential-build.json "$RUNNER_TEMP/n8n-whole-corpus-workflow/producer-build.json"
