#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/09-eslint-plugin/scripts/qualify-packed-public-entries-in-an-isolated-consumer.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
vp exec pnpm --dir npm/linter/oxlint pack --pack-destination "$VIZE_ESLINT_EVIDENCE"
vp exec pnpm --dir npm/linter/eslint pack --pack-destination "$VIZE_ESLINT_EVIDENCE"
export VIZE_ESLINT_PACKED_CONSUMER="$RUNNER_TEMP/eslint-packed-$SOURCE_SHA"
mkdir -p "$VIZE_ESLINT_PACKED_CONSUMER"
node "$VIZE_CI_FILES/qualify-packed-public-entries-in-an-isolated-consumer.ts"
cp npm/linter/eslint/test/packed-consumer.ts "$VIZE_ESLINT_PACKED_CONSUMER/consumer.ts"
cp npm/linter/eslint/test/real-projects.ts "$VIZE_ESLINT_PACKED_CONSUMER/real-projects.ts"
export VIZE_ESLINT_PROJECT_ROOT="$GITHUB_WORKSPACE"
export VIZE_ESLINT_PROJECT_EVIDENCE="$VIZE_ESLINT_EVIDENCE/real-projects.json"
(cd "$VIZE_ESLINT_PACKED_CONSUMER" && node --experimental-strip-types consumer.ts) > "$VIZE_ESLINT_EVIDENCE/packed-consumer.json"
cp "$VIZE_ESLINT_PACKED_CONSUMER/package.json" "$VIZE_ESLINT_EVIDENCE/packed-package.json"
cp "$VIZE_ESLINT_PACKED_CONSUMER/pnpm-lock.yaml" "$VIZE_ESLINT_EVIDENCE/packed-pnpm-lock.yaml"
node "$VIZE_CI_FILES/qualify-packed-public-entries-in-an-isolated-consumer-2.ts"
