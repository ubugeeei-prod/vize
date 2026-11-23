#!/usr/bin/env bash
gh api "repos/$GITHUB_REPOSITORY/compare/$REQUIRED_ICON_SOURCE...$PUBLIC_CANDIDATE" > "$RUNNER_TEMP/public-release-proof/icons-source-comparison.json"
node "$VIZE_CI_FILES/prove-the-candidate-contains-the-required-icon-source.ts"
