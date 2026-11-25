#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/resolve-the-requested-version-with-current-workflow-tooling.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir -p "$RUNNER_TEMP/release-quota-tooling"
git archive "$GITHUB_SHA" package.json tools/support/release tools/support/archive tests/tooling/release/quota-live-legacy-refusal.ts | tar -x -C "$RUNNER_TEMP/release-quota-tooling"
quota_main="$RUNNER_TEMP/release-quota-tooling/tools/support/release/release/main.ts"
version="$RELEASE_VERSION"
if [[ -z "$version" ]]; then
  version="$(node "$quota_main" next)"
fi
echo "version=$version" >> "$GITHUB_OUTPUT"
