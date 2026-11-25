#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/e2e/validate-optional-target-sha.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ ! "$REQUESTED_TARGET_SHA" =~ ^[0-9a-f]{40}$ ]]; then
  echo "::error title=Invalid target_sha::target_sha must be a full lowercase 40-character commit SHA"
  exit 1
fi
if [[ "$RUN_HEAD_SHA" != "$REQUESTED_TARGET_SHA" ]]; then
  echo "::error title=Dispatch ref mismatch::--ref must point at $REQUESTED_TARGET_SHA (got $RUN_HEAD_SHA)"
  exit 1
fi
