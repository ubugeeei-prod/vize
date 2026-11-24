#!/usr/bin/env bash
[[ "$CANDIDATE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$RECEIPT_RUN_ID" =~ ^[1-9][0-9]*$ ]]
IFS=$'\t' read -r run source attempt < <(
  gh api "repos/$GITHUB_REPOSITORY/actions/runs/$RECEIPT_RUN_ID" \
    --jq "$(<"$VIZE_CI_FILES/candidate-fields.jq")"
)
test "$run" = "$RECEIPT_RUN_ID"
test "$source" = "$CANDIDATE_SHA"
[[ "$attempt" =~ ^[1-9][0-9]*$ ]]
printf 'run=%s\nattempt=%s\n' "$run" "$attempt" >> "$GITHUB_OUTPUT"
