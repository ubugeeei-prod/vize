#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends linux-tools-generic
sudo sysctl -w kernel.perf_event_paranoid=-1 kernel.kptr_restrict=0
perf_bin=""
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then perf_bin="$candidate"; break; fi
done
test -n "$perf_bin"
mkdir -p profiles
input="$PWD/tools/benchmarks/scripts/__in__"
for v in $COMPILE_BENCH_PROFILE_VERSIONS; do
  node_file=$(ls "$RUNNER_TEMP/native/$v"/package/*.node)
  spec=$(printf '{"label":"%s","kind":"native","path":"%s","mode":"loop1t"}' "$v" "$node_file")
  "$perf_bin" stat -e task-clock,instructions,cycles,cache-misses,branch-misses -o "profiles/$v.stat.txt" \
    node tools/benchmarks/scripts/compile-history.ts --child "$spec" --input "$input" --runs 5 --warmups 1 > /dev/null
  "$perf_bin" record -F 999 --call-graph dwarf,16384 -o "profiles/$v.perf.data" \
    node tools/benchmarks/scripts/compile-history.ts --child "$spec" --input "$input" --runs 3 --warmups 1 > /dev/null
  "$perf_bin" report -i "profiles/$v.perf.data" --no-children --sort dso,symbol --stdio --percent-limit 0.3 2> /dev/null | head -150 > "profiles/$v.self.txt"
  "$perf_bin" report -i "profiles/$v.perf.data" --children --sort symbol --stdio -g none --percent-limit 1 2> /dev/null | head -200 > "profiles/$v.children.txt"
  "$perf_bin" script -i "profiles/$v.perf.data" 2> /dev/null | gzip > "profiles/$v.perf-script.txt.gz"
  echo "::group::$v perf stat"; cat "profiles/$v.stat.txt"; echo "::endgroup::"
  echo "::group::$v self"; cat "profiles/$v.self.txt"; echo "::endgroup::"
  echo "::group::$v children"; cat "profiles/$v.children.txt"; echo "::endgroup::"
  rm "profiles/$v.perf.data"
done
