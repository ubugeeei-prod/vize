#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$OBSERVATION_ROOT"
for role in base head; do
  if [[ "$role" = base ]]; then sha="$BASE_SHA"; else sha="$SOURCE_SHA"; fi
  target="${RUNNER_TEMP}/${role}-target"
  git checkout --detach "$sha"
  test "$(git rev-parse HEAD)" = "$sha"
  rustc -Vv > "$OBSERVATION_ROOT/${role}-rustc.txt"
  CARGO_TARGET_DIR="$target" cargo build --locked --profile ci-opt -p vize --features profiling --message-format=json-render-diagnostics > "$OBSERVATION_ROOT/${role}-build.jsonl" 2> "$OBSERVATION_ROOT/${role}-build.stderr"
  vp exec node tools/support/compat/davinci/lsp-build-identity.ts "$target/ci-opt/vize" ci-opt "$OBSERVATION_ROOT/${role}-build.jsonl" "$OBSERVATION_ROOT/${role}-build-identity.json"
done
git checkout --detach "$SOURCE_SHA"
