#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends linux-tools-generic
sudo sysctl -w kernel.perf_event_paranoid=-1
perf_bin=""
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then perf_bin="$candidate"; break; fi
done
test -n "$perf_bin"
input="$PWD/tools/benchmarks/scripts/__in__"
{
  echo
  echo "perf stat, one process: 1 warmup + 5 passes"
  echo
  echo "| Build | Mode | Instructions | Cycles | Task clock |"
  echo "| --- | --- | ---: | ---: | ---: |"
} >> compile-compare.md
for mode in loop1t batch1t batchmax; do
  for side in base head; do
    spec=$(printf '{"label":"%s","kind":"native","path":"%s","mode":"%s"}' "$side" "$RUNNER_TEMP/$side.node" "$mode")
    "$perf_bin" stat -x, -e instructions,cycles,task-clock -o "stat-$side-$mode.csv" \
      node tools/benchmarks/scripts/compile-history.ts --child "$spec" --input "$input" --runs 5 --warmups 1 > /dev/null
    node "$VIZE_CI_FILES/count-instructions.cjs" "stat-$side-$mode.csv" "$side" "$mode" >> compile-compare.md
  done
done
cat compile-compare.md
