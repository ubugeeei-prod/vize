#!/usr/bin/env bash
set -euo pipefail
for role in base head; do
  source="$GITHUB_WORKSPACE"
  if [[ "$role" == base ]]; then source="$RUNNER_TEMP/recovery-allocation-base"; fi
  cargo +1.100.0 build --locked --profile ci-opt --manifest-path "$source/Cargo.toml" --target-dir "$GITHUB_WORKSPACE/target/recovery-allocation-$role-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT" -p vize_l0 --example source_recovery_clone_benchmark --features micro-profiling --message-format json > "allocation-evidence/profile-build-$role.jsonl"
  cp "target/recovery-allocation-$role-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT/ci-opt/examples/source_recovery_clone_benchmark" "allocation-evidence/instrumented-$role"
  for scenario in irreparable valid repairable; do
    "allocation-evidence/instrumented-$role" 8 4096 profile "$scenario" > "allocation-evidence/profile-$scenario-$role-4096.json"
  done
done
node tools/benchmarks/scripts/compare_recovery_clone.ts allocation-tiny allocation-evidence
