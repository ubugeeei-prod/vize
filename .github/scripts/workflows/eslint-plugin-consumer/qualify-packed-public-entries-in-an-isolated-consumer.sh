#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/eslint-plugin-consumer/qualify-packed-public-entries-in-an-isolated-consumer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
vp exec pnpm --dir npm/linter/oxlint pack --pack-destination "$VIZE_ESLINT_EVIDENCE"
vp exec pnpm --dir npm/linter/eslint pack --pack-destination "$VIZE_ESLINT_EVIDENCE"
export VIZE_ESLINT_PACKED_CONSUMER="$RUNNER_TEMP/eslint-packed-$SOURCE_SHA"
mkdir -p "$VIZE_ESLINT_PACKED_CONSUMER"
node "$VIZE_CI_FILES/qualify-packed-public-entries-in-an-isolated-consumer.ts"
cp npm/linter/eslint/test/packed-consumer.mjs "$VIZE_ESLINT_PACKED_CONSUMER/consumer.mjs"
(cd "$VIZE_ESLINT_PACKED_CONSUMER" && node consumer.mjs) > "$VIZE_ESLINT_EVIDENCE/packed-consumer.json"
cp "$VIZE_ESLINT_PACKED_CONSUMER/package.json" "$VIZE_ESLINT_EVIDENCE/packed-package.json"
cp "$VIZE_ESLINT_PACKED_CONSUMER/pnpm-lock.yaml" "$VIZE_ESLINT_EVIDENCE/packed-pnpm-lock.yaml"
node "$VIZE_CI_FILES/qualify-packed-public-entries-in-an-isolated-consumer-2.ts"
