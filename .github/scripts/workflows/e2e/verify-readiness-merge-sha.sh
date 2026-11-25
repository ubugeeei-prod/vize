#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/e2e/verify-readiness-merge-sha.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
test "$(git rev-parse HEAD)" = "$EXPECTED_SHA"
echo "Tested merge SHA $EXPECTED_SHA for source head $SOURCE_HEAD_SHA" >> "$GITHUB_STEP_SUMMARY"
