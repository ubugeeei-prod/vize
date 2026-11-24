#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/published-fresh-project/select-an-exact-version-and-verifier-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
version="$REQUESTED_VERSION"
if [[ -z "$version" ]]; then
  version=$(npm view --registry=https://registry.npmjs.org vize dist-tags.latest --json |
    node "$VIZE_CI_FILES/select-an-exact-version-and-verifier-source.cjs")
fi
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
source_ref="$REQUESTED_SOURCE"
if [[ -z "$source_ref" ]]; then
  source_ref="${PULL_REQUEST_HEAD:-$EVENT_SHA}"
fi
[[ -n "$source_ref" ]]
printf 'version=%s\nsource_ref=%s\n' "$version" "$source_ref" >> "$GITHUB_OUTPUT"
