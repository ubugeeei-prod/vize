#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/compile-bench/profile-source-compile.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
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
node_file=$(ls "$PWD"/npm/native/*.node)
input="$PWD/tools/benchmarks/scripts/__in__"
for mode in loop1t batchmax; do
  spec=$(printf '{"label":"source","kind":"native","path":"%s","mode":"%s"}' "$node_file" "$mode")
  "$perf_bin" stat -e task-clock,instructions,cycles,cache-misses -o "profiles/$mode.stat.txt" \
    node tools/benchmarks/scripts/compile-history.ts --child "$spec" --input "$input" --runs 5 --warmups 1 > /dev/null
  "$perf_bin" record -F 2999 --call-graph fp -o "profiles/$mode.perf.data" \
    node tools/benchmarks/scripts/compile-history.ts --child "$spec" --input "$input" --runs 5 --warmups 1 > /dev/null
  "$perf_bin" report -i "profiles/$mode.perf.data" --no-children --sort dso,symbol --stdio -g none --percent-limit 0.2 2> /dev/null | head -250 > "profiles/$mode.self.txt"
  "$perf_bin" report -i "profiles/$mode.perf.data" --children --sort symbol --stdio -g none --percent-limit 0.3 2> /dev/null | head -400 > "profiles/$mode.children.txt"
  "$perf_bin" script -i "profiles/$mode.perf.data" 2> /dev/null | gzip > "profiles/$mode.perf-script.txt.gz"
  for f in stat self children; do echo "::group::$mode $f"; head -120 "profiles/$mode.$f.txt"; echo "::endgroup::"; done
  rm "profiles/$mode.perf.data"
done
