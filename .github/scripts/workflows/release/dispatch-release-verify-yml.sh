#!/usr/bin/env bash
gh workflow run release-verify.yml --repo "$GITHUB_REPOSITORY" --ref main \
  -f version="$RELEASE_VERSION" -f tag="$RELEASE_TAG" -f sha="$RELEASE_SHA" \
  -f packages="$RELEASE_PACKAGES"
