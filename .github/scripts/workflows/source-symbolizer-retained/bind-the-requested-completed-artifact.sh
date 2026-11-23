#!/usr/bin/env bash
set -euo pipefail
[[ "$SOURCE_RUN" =~ ^[0-9]+$ && "$SOURCE_ARTIFACT" =~ ^[0-9]+$ ]]
gh api "repos/$GITHUB_REPOSITORY/actions/artifacts/$SOURCE_ARTIFACT" > "$RUNNER_TEMP/source-artifact-api.json"
node "$VIZE_CI_FILES/verify-artifact-identity.ts"
