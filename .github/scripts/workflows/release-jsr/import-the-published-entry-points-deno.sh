#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-jsr/import-the-published-entry-points-deno.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cp "$GITHUB_WORKSPACE/tools/support/release/jsr/consumer-deno.json" deno.json
deno run --config deno.json --allow-all consumer.ts "jsr:@vizejs/vize@${JSR_VERSION#v}"
