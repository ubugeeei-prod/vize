#!/usr/bin/env bash
set -euo pipefail
base=$(gh api "repos/$GITHUB_REPOSITORY/compare/$TARGET_SHA...$HEAD_SHA" --jq .merge_base_commit.sha)
[[ "$base" =~ ^[a-f0-9]{40}$ ]]
git fetch --no-tags --depth=1 origin "$base" "$HEAD_SHA"
node tools/commands/levels/codegen-equivalence.ts "$base" "$HEAD_SHA" vize bin:vize "$MODE" "$RUNNER_TEMP/cli.json"
status=0
node tools/commands/levels/codegen-equivalence.ts "$base" "$HEAD_SHA" vize_vitrine node-addon "$MODE" "$RUNNER_TEMP/addon.json" --control-plugin-identity || status=$?
if ((status != 0)); then
  test "$status" = 1
  node tools/commands/levels/native-addon-identity.ts "$RUNNER_TEMP/addon.json"
fi
node tools/support/compat/npm/bundler-block-products.ts --stock "$RUNNER_TEMP/addon.json"
