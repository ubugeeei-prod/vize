#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/real-project-matrix/select-and-hydrate-fixture-shard.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mapfile -t fixture_paths < <(
  node tools/support/compat/fixtures/tool-matrix-report.ts \
    --list-fixture-paths \
    --shard-index "$FIXTURE_SHARD_INDEX" \
    --shard-count "$FIXTURE_SHARD_COUNT"
)
if (( ${#fixture_paths[@]} == 0 )); then
  echo "::error title=Empty fixture shard::shard $FIXTURE_SHARD_INDEX selected no projects"
  exit 1
fi
mkdir -p "$FIXTURE_REPORT_DIR"
printf '%s\n' "${fixture_paths[@]}" > "$FIXTURE_REPORT_DIR/selected-fixtures.txt"
git submodule update --init --depth 1 -- "${fixture_paths[@]}"
