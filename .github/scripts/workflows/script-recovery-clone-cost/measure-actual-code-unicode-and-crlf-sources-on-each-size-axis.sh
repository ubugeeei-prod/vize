#!/usr/bin/env bash
set -euo pipefail
cpus=$(cat evidence/affinity.txt)
for scenario in irreparable valid repairable; do
  for bytes in 131072 1048576 10485760; do
    for round in 1 2 3; do
      order='base head'
      if [[ "$round" == 2 ]]; then order='head base'; fi
      for role in $order; do
        /usr/bin/time -v -o "evidence/time-$scenario-$role-$bytes-$round.txt" taskset -c "$cpus" "evidence/normal-$role" 5 "$bytes" normal "$scenario" > "evidence/normal-$scenario-$role-$bytes-$round.json"
      done
    done
    for role in base head; do
      taskset -c "$cpus" "evidence/instrumented-$role" 3 "$bytes" profile "$scenario" > "evidence/profile-$scenario-$role-$bytes.json"
    done
  done
done
node tools/benchmarks/scripts/compare_recovery_clone.ts matrix evidence
rm evidence/normal-base evidence/normal-head evidence/instrumented-base evidence/instrumented-head
