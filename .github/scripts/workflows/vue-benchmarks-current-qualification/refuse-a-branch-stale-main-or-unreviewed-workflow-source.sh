#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vue-benchmarks-current-qualification/refuse-a-branch-stale-main-or-unreviewed-workflow-source.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
[[ "$EXPECTED_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$GITHUB_REF" = refs/heads/main
test "$EXECUTED_WORKFLOW_SHA" = "$EXPECTED_SOURCE_SHA"
