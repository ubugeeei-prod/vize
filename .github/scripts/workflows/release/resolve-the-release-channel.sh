#!/usr/bin/env bash
npm_tag="$(node tools/commands/release/npm/tag.ts "$RELEASE_VERSION")"
echo "npm_tag=$npm_tag" >> "$GITHUB_OUTPUT"
if [[ "$npm_tag" == "latest" ]]; then
  echo "prerelease=false" >> "$GITHUB_OUTPUT"
  echo "make_latest=true" >> "$GITHUB_OUTPUT"
else
  echo "prerelease=true" >> "$GITHUB_OUTPUT"
  echo "make_latest=false" >> "$GITHUB_OUTPUT"
fi
