#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-jsr/require-a-full-source-commit.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
# The source is the dispatched commit unless a full commit SHA names another.
if [[ -n "$JSR_SOURCE_SHA" && ! "$JSR_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]; then
  echo "source_sha must be a full 40-character lowercase commit SHA" >&2
  exit 1
fi
