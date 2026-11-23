#!/usr/bin/env bash
[[ "$EXPECTED_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$GITHUB_REF" = refs/heads/main
test "$EXECUTED_WORKFLOW_SHA" = "$EXPECTED_SOURCE_SHA"
