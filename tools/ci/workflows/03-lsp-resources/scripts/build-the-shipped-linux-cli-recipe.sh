#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-lsp-resources/scripts/build-the-shipped-linux-cli-recipe.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo build --release -p vize --target x86_64-unknown-linux-gnu --message-format=json-render-diagnostics > "${RUNNER_TEMP}/release-build.jsonl"
