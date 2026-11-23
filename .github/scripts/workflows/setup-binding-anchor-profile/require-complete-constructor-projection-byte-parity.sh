#!/usr/bin/env bash
set -euo pipefail
for role in base head; do
  source_root="$GITHUB_WORKSPACE"
  if [[ "$role" == base ]]; then source_root="$RUNNER_TEMP/binding-base"; fi
  (cd "$source_root" && CARGO_TARGET_DIR="$RUNNER_TEMP/binding-$role-target" VIZE_CONSTRUCTOR_PROJECTION_CAPTURE="$RUNNER_TEMP/binding-evidence/$role-constructor.json" cargo test --locked --profile ci-opt -p vize_canon --test constructor_emission_projection complete_constructor_projection_packets -- --exact --nocapture) > "$RUNNER_TEMP/binding-evidence/$role-constructor.stdout" 2> "$RUNNER_TEMP/binding-evidence/$role-constructor.stderr"
done
node tools/benchmarks/scripts/setup-binding-constructor-evidence.ts "$RUNNER_TEMP/binding-evidence" "$RUNNER_TEMP/binding-base" "$GITHUB_WORKSPACE"
