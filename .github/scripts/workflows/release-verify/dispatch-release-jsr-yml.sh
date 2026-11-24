#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-verify/dispatch-release-jsr-yml.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
gh workflow run release-jsr.yml --repo "$GITHUB_REPOSITORY" --ref main \
  -f version="$JSR_VERSION" -f source_sha="$JSR_SOURCE"
