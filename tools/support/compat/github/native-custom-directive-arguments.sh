#!/usr/bin/env bash
# Qualify actual custom argument expressions and unchanged built-in controls.
set -euo pipefail
[[ "$GITHUB_EVENT_NAME" == pull_request ]] || exit 0
unset VIZE_TEST_DISABLE_TSGO
export VIZE_TEST_REQUIRE_TSGO=1
capture="$RUNNER_TEMP/native-source-recipe-custom-directive-arguments"
mkdir -p "$capture"
cd "$NATIVE_PHASE_SOURCE_ROOT"
baseline=67e3d0e6775cc3a1f29bfb9309d991252f9e75ff
git cat-file -e "$baseline^{commit}" || git fetch --no-tags origin "$baseline"
paths=(
  crates/vize_canon/src/virtual_ts/expressions/directive_values.rs
  crates/vize_canon/src/virtual_ts/expressions/directive_values/arguments.rs
  crates/vize_canon/src/virtual_ts/expressions/directive_values/presence.rs
  crates/vize_canon/src/virtual_ts/expressions/statements.rs
)
restore() {
  for path in "${paths[@]}"; do cp "$capture/candidate-source/$path" "$path"; done
}
qualify() {
  mkdir -p "$VIZE_CUSTOM_DIRECTIVE_CAPTURE"
  cargo test --locked --profile ci-opt -p vize_canon \
    --test native_custom_directive_arguments "$@" \
    --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' \
    --config 'profile.ci-opt.codegen-units=16' -- --nocapture --test-threads=1 \
    2>&1 | tee "$VIZE_CUSTOM_DIRECTIVE_CAPTURE/runtime.log"
}
printf '%s\n' "$baseline" > "$capture/baseline-source-sha.txt"
candidate="$(git rev-parse HEAD)"
printf '%s\n' "$candidate" > "$capture/candidate-source-sha.txt"
git diff --exit-code HEAD --
rustc --version --verbose > "$capture/toolchain.txt"
for path in "${paths[@]}"; do
  mkdir -p "$capture/candidate-source/$(dirname "$path")" "$capture/baseline-source/$(dirname "$path")"
  cp "$path" "$capture/candidate-source/$path"
  if git cat-file -e "$baseline:$path" 2>/dev/null; then git show "$baseline:$path" > "$capture/baseline-source/$path"; fi
done
trap restore EXIT
for path in "${paths[@]}"; do
  if [[ -f "$capture/baseline-source/$path" ]]; then cp "$capture/baseline-source/$path" "$path"; else rm "$path"; fi
done
python3 tools/support/compat/github/custom-directive-source-custody.py "$baseline" "$candidate" baseline "$capture/baseline-source-manifest.json"
export VIZE_CUSTOM_DIRECTIVE_CAPTURE="$capture/baseline" VIZE_CUSTOM_DIRECTIVE_BASELINE=1
qualify
restore
git diff --exit-code HEAD --
[[ "$(git rev-parse HEAD)" == "$candidate" ]]
python3 tools/support/compat/github/custom-directive-source-custody.py "$baseline" "$candidate" candidate "$capture/candidate-source-manifest.json"
unset VIZE_CUSTOM_DIRECTIVE_BASELINE
export VIZE_CUSTOM_DIRECTIVE_CAPTURE="$capture/candidate"
qualify --test native_spread_diagnostics
python3 - "$capture" <<'PY'
import json
from pathlib import Path
import sys
root = Path(sys.argv[1])
before = json.loads((root / 'baseline/baseline.json').read_text())
after = json.loads((root / 'candidate/original.json').read_text())
for field in ('native', 'nativeSha256', 'source', 'configuration', 'originalCustody'):
    assert before[field] == after[field], field
before = json.loads((root / 'baseline-source-manifest.json').read_text())
after = json.loads((root / 'candidate-source-manifest.json').read_text())
assert before['buildEnvironment'] == after['buildEnvironment']
assert before['neutralDriverManifest'] == after['neutralDriverManifest']
PY
