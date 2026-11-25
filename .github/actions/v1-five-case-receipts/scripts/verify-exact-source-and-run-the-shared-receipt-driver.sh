#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/v1-five-case-receipts/scripts/verify-exact-source-and-run-the-shared-receipt-driver.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
mkdir -p .qualification/v1-five-case
if [ "$OPERATION" = plan ]; then
  output=.qualification/v1-five-case/plan.json
  node tools/commands/release/v1-five-case-receipts.ts plan \
    --source "$SOURCE_SHA" \
    --candidate "$CANDIDATE_SHA" \
    --mode "$MODE" \
    --run "$RUN_IDENTITY" \
    --attempt "$RUN_ATTEMPT" \
    --shards "$SHARD_COUNT" \
    --output "$output"
elif [ "$OPERATION" = validate ]; then
  test -n "$BUNDLES_DIRECTORY"
  test -d "$BUNDLES_DIRECTORY"
  output=$BUNDLES_DIRECTORY
  node tools/commands/release/v1-five-case-receipts.ts validate-bundles \
    --source "$SOURCE_SHA" \
    --candidate "$CANDIDATE_SHA" \
    --mode "$MODE" \
    --bundles "$BUNDLES_DIRECTORY"
elif [ "$OPERATION" = admit ]; then
  test -n "$BUNDLES_DIRECTORY"
  test -d "$BUNDLES_DIRECTORY"
  output=.qualification/v1-five-case/admission.json
  args=(
    admit
    --source "$SOURCE_SHA"
    --candidate "$CANDIDATE_SHA"
    --mode "$MODE"
    --bundles "$BUNDLES_DIRECTORY"
    --output "$output"
  )
  if [ "$ENFORCE_COMPLETE" = true ]; then args+=(--enforce-complete); fi
  node tools/commands/release/v1-five-case-receipts.ts "${args[@]}"
else
  echo "operation must be plan, validate, or admit" >&2
  exit 1
fi
echo "output_path=$output" >> "$GITHUB_OUTPUT"
