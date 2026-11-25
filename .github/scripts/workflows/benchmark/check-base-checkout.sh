#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/benchmark/check-base-checkout.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$EVENT_NAME" != "pull_request" ]] && \
   ! git -C "$GITHUB_WORKSPACE/head" merge-base --is-ancestor "$BASE_SHA" "$HEAD_SHA"; then
  echo "::error title=Invalid benchmark range::base_sha must be an ancestor of head_sha"
  exit 1
fi
if git grep -n -E '^(<<<<<<<|=======$|>>>>>>>)' -- . ':(exclude)tests/_fixtures/_git/**'; then
  echo "skip=true" >> "$GITHUB_OUTPUT"
  {
    echo "## PR Benchmark"
    echo
    echo "Skipped because the base checkout contains unresolved merge conflict markers."
    echo
    echo "- Base: \`$BASE_SHA\`"
    echo "- Head: \`$HEAD_SHA\`"
  } > "$GITHUB_WORKSPACE/benchmark-summary.md"
  printf '{"skipped":true,"reason":"base_has_conflict_markers","base":"%s","head":"%s"}\n' "$BASE_SHA" "$HEAD_SHA" > "$GITHUB_WORKSPACE/benchmark-results.json"
else
  echo "skip=false" >> "$GITHUB_OUTPUT"
fi
