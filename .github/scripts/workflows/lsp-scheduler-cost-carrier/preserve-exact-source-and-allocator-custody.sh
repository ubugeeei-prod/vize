#!/usr/bin/env bash
set -euo pipefail
test "$(git rev-parse HEAD)" = "$SOURCE_SHA"
mkdir scheduler-evidence
base=b34edc288fda5f596eea16183b225cd894fd2c3a
git merge-base --is-ancestor "$base" HEAD
test "$(git show "$base:crates/vize_maestro/src/server/initial_diagnostics/pending.rs" | git hash-object --stdin)" = 06f98a9c70c884c0b19ed032560573f78e23a1eb
if git grep -n '#\[global_allocator\]' "$base" -- crates/vize_maestro/src; then
  echo "An existing Maestro allocator requires a new isolation audit" >&2
  exit 1
else
  result=$?
  test "$result" -eq 1
fi
git diff "$base" HEAD -- crates/vize_maestro > scheduler-evidence/test-instrumentation.diff
git status --porcelain > scheduler-evidence/source-status.txt
git show -s --format=fuller HEAD > scheduler-evidence/commit.txt
rustc -vV > scheduler-evidence/rustc.txt
nproc > scheduler-evidence/cpus.txt
target_dir="$RUNNER_TEMP/indexed-scheduler-$SOURCE_SHA"
test ! -e "$target_dir"
echo "CARGO_TARGET_DIR=$target_dir" >> "$GITHUB_ENV"
