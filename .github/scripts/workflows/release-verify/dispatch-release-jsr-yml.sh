#!/usr/bin/env bash
gh workflow run release-jsr.yml --repo "$GITHUB_REPOSITORY" --ref main \
  -f version="$JSR_VERSION" -f source_sha="$JSR_SOURCE"
