#!/usr/bin/env bash
set -euo pipefail
baseline_sha=49dc7684234a6ae723e1201c3d2bcb7c0039dbac
baseline_source="$RUNNER_TEMP/diagnostic-capacity-before-source"
baseline_target="$RUNNER_TEMP/diagnostic-capacity-before-target"
capture="$GITHUB_WORKSPACE/target/vize-tests/diagnostic-capacity-129"
control=crates/vize_maestro/src/server/initial_diagnostics/tests/conservation_tests.rs
tests=crates/vize_maestro/src/server/initial_diagnostics/tests.rs
mkdir -p "$capture"
test ! -e "$baseline_source"
test ! -e "$baseline_target"
git fetch --no-tags --depth=1 origin "$baseline_sha"
git worktree add --detach "$baseline_source" "$baseline_sha"
test "$(git -C "$baseline_source" rev-parse HEAD)" = "$baseline_sha"
test "$(git -C "$baseline_source" hash-object crates/vize_maestro/src/server/initial_diagnostics.rs)" = 8e1bc289fc3e7cc32674764aff0581a46b7908dc
mkdir -p "$baseline_source/${control%/*}"
cp "$control" "$baseline_source/$control"
printf '\nmod conservation_tests;\n' >> "$baseline_source/$tests"
sha256sum "$control" "$baseline_source/$control" > "$capture/conservation-control.sha256"
git -C "$baseline_source" diff -- "$tests" > "$capture/before-test-carrier.diff"
git -C "$baseline_source" status --porcelain > "$capture/before-test-carrier.status"
if (
  cd "$baseline_source"
  CARGO_TARGET_DIR="$baseline_target" cargo test --locked --profile ci -p vize_maestro --lib server::initial_diagnostics::tests::conservation_tests::every_open_uri_retains_its_latest_job_at_65_129_and_2500_documents -- --exact
) > "$capture/before-scheduler.log" 2>&1
then
  cat "$capture/before-scheduler.log"
  echo "The original source unexpectedly passed the conservation law" >&2
  exit 1
else
  result=$?
  test "$result" -eq 101
fi
cat "$capture/before-scheduler.log"
node "$VIZE_CI_FILES/verify-original-scheduler-failure.ts" "$capture/before-scheduler.log"
