#!/usr/bin/env bash
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y linux-tools-generic binutils llvm
shopt -s nullglob
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then
    echo "REPLAY_PERF=$candidate" >> "$GITHUB_ENV"
    break
  fi
done
