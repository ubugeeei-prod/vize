#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends linux-tools-generic libc6-dbg
# Kernel frames and symbols, and call chains deep enough for checker recursion.
sudo sysctl -w kernel.perf_event_paranoid=-1 kernel.kptr_restrict=0 kernel.perf_event_max_stack=1024
perf_bin=""
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then perf_bin="$candidate"; break; fi
done
test -n "$perf_bin"
"$perf_bin" --version
echo "PERF_BIN=$perf_bin" >> "$GITHUB_ENV"
mkdir -p "$RUNNER_TEMP/flamegraph"
for script in flamegraph.pl difffolded.pl; do
  curl -fsSL "https://raw.githubusercontent.com/brendangregg/FlameGraph/$FLAMEGRAPH_SHA/$script" -o "$RUNNER_TEMP/flamegraph/$script"
done
