#!/usr/bin/env bash
bench_cpu_set=$(node "$VIZE_CI_FILES/../shared/runner-resources.ts" first 8)
printf '%s\n' "$bench_cpu_set" > "$RUNNER_TEMP/lsp-marker-evidence/affinity.txt"
for round in 1 2 3; do
  order='base head'
  if [[ "$round" == 2 ]]; then order='head base'; fi
  for role in $order; do
    taskset -c "$bench_cpu_set" node tools/support/compat/davinci/lsp-resources.ts --server "$RUNNER_TEMP/lsp-marker-$role-normal" --files "$FILES" --warm-providers 16 --runs 1 --idle-seconds 10 --build 'ci-opt default features; normal allocator; setup excluded' --out "$RUNNER_TEMP/lsp-marker-evidence/$role-normal-$round.json"
  done
done
