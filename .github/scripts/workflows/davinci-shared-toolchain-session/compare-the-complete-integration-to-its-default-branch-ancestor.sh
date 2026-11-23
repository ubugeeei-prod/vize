#!/usr/bin/env bash
set -euo pipefail
base=$(gh api "repos/$GITHUB_REPOSITORY/compare/$DEFAULT_BRANCH...$HEAD_SHA" --jq "$(<"$VIZE_CI_FILES/merge-base-sha.jq")")
[[ "$base" =~ ^[a-f0-9]{40}$ ]]
git fetch --no-tags --depth=1 origin "$base" "$HEAD_SHA"
node tools/commands/levels/codegen-equivalence.ts "$base" "$HEAD_SHA" vize bin:vize "$MODE" "$RUNNER_TEMP/cli.json"
VIZE_CODEGEN_PROBE=1 node --test tests/tooling/native-addon-identity.test.ts
status=0
node tools/commands/levels/codegen-equivalence.ts "$base" "$HEAD_SHA" vize_vitrine node-addon "$MODE" "$RUNNER_TEMP/addon.json" --control-plugin-identity || status=$?
test "$status" = 1
node tools/commands/levels/native-addon-identity.ts "$RUNNER_TEMP/addon.json" | tee "$RUNNER_TEMP/identity.json"
node tools/support/compat/npm/bundler-block-products.ts --stock "$RUNNER_TEMP/addon.json"
