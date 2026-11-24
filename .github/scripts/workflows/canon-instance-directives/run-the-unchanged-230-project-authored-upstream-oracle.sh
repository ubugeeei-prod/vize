#!/usr/bin/env bash
set -euo pipefail
ln -s "$RUNNER_TEMP/instance-directive-provider/node_modules" "$GITHUB_WORKSPACE/tests/node_modules"
native_path=$(node "$VIZE_CI_FILES/run-the-unchanged-230-project-authored-upstream-oracle.ts")
"$RUNNER_TEMP/instance-directives/binaries/head/vize" --version
set +e
VIZE_LSP_BIN="$RUNNER_TEMP/instance-directives/binaries/head/vize" VIZE_LSP_REQUIRE_SOURCE_BUILD=0 CORSA_PATH="$native_path" node --test --test-reporter=spec tests/tooling/canon-upstream-diagnostics.test.ts > "$RUNNER_TEMP/instance-directives/upstream-230.txt" 2>&1
upstream_status=$?
set -e
if [[ -d "$GITHUB_WORKSPACE/target/differential/fallthrough-listener-inputs" ]]; then
  cp -a "$GITHUB_WORKSPACE/target/differential/fallthrough-listener-inputs" "$RUNNER_TEMP/instance-directives/upstream-inputs"
fi
cat "$RUNNER_TEMP/instance-directives/upstream-230.txt"
exit "$upstream_status"
