#!/usr/bin/env bash
set -euo pipefail
mkdir -p evidence
test "$(git -C source rev-parse HEAD)" = "$SOURCE_SHA"
test "$(git -C driver rev-parse HEAD)" = "$DRIVER_SHA"
input="$GITHUB_WORKSPACE/driver/tests/fuzz/regressions/js_ts_expression/issue-10401.input"
echo "6d668844f745732cdc92bdafa21f72ba5d09d9c14305d4ea9174fb802493dcc8  $input" | sha256sum -c -
mkdir -p source/crates/vize_atelier_core/examples
cp driver/crates/vize_atelier_core/examples/fuzz_expression_10401_profile.rs source/crates/vize_atelier_core/examples/
{
  printf 'source_sha=%s\ndriver_sha=%s\nrole=%s\n' "$SOURCE_SHA" "$DRIVER_SHA" "$ROLE"
  rustc +nightly-2026-10-23 -Vv
  uname -a
  sha256sum "$input" driver/crates/vize_atelier_core/examples/fuzz_expression_10401_profile.rs
  git -C source status --short
} > evidence/provenance.txt
target="$RUNNER_TEMP/expression-10401-$ROLE-$SOURCE_SHA-optimized"
test ! -e "$target"
echo "CARGO_TARGET_DIR=$target" >> "$GITHUB_ENV"
echo "EXPRESSION_INPUT=$input" >> "$GITHUB_ENV"
