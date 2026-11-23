#!/usr/bin/env bash
set -euo pipefail
for scenario in irreparable valid repairable; do
  for bytes in 131072 1048576 10485760; do
    for role in base head; do
      "allocation-evidence/instrumented-$role" 3 "$bytes" profile "$scenario" > "allocation-evidence/profile-$scenario-$role-$bytes.json"
    done
  done
done
node tools/benchmarks/scripts/compare_recovery_clone.ts allocation-matrix allocation-evidence
rm allocation-evidence/instrumented-base allocation-evidence/instrumented-head
