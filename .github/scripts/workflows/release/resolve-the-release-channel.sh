#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release/resolve-the-release-channel.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
npm_tag="$(node tools/commands/release/npm/tag.ts "$RELEASE_VERSION")"
echo "npm_tag=$npm_tag" >> "$GITHUB_OUTPUT"
if [[ "$npm_tag" == "latest" ]]; then
  echo "prerelease=false" >> "$GITHUB_OUTPUT"
  echo "make_latest=true" >> "$GITHUB_OUTPUT"
else
  echo "prerelease=true" >> "$GITHUB_OUTPUT"
  echo "make_latest=false" >> "$GITHUB_OUTPUT"
fi
