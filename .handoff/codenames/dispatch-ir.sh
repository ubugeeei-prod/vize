#!/bin/bash
# Waits for the CI queue to drop below 40, then dispatches the release IR identity run.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/90a0aa1d-5403-4093-b702-6ace0f463fd6/scratchpad/codenames
cd "$1" || exit 2
BASE=$(git rev-parse 4a0fb417ed); HEAD_SHA=$2
until [ "$(gh run list --status queued --limit 300 --json databaseId --jq length 2>/dev/null || echo 999)" -lt 40 ]; do sleep 60; done
gh workflow run level-instruction-counts.yml --ref bench/levels-ir \
  -f base_sha="$BASE" -f head_sha="$HEAD_SHA" -f args="$(cat "$S/ir-args.txt")"
sleep 15
gh run list --workflow level-instruction-counts.yml --branch bench/levels-ir --limit 1 --json databaseId --jq '.[0].databaseId'
