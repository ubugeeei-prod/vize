#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y linux-tools-generic binutils llvm elfutils
sudo sysctl kernel.perf_event_paranoid=0
shopt -s nullglob
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then
    echo "SMOKE_PERF=$candidate" >> "$GITHUB_ENV"
    break
  fi
done
