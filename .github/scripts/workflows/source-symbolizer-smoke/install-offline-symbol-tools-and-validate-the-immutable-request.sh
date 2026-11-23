#!/usr/bin/env bash
set -euo pipefail
[[ "$REPLAY_RUN_ID" =~ ^[0-9]+$ && "$REPLAY_ARTIFACT_ID" =~ ^[0-9]+$ ]]
sudo apt-get update -qq
sudo apt-get install -y linux-tools-generic binutils
shopt -s nullglob
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then
    echo "SMOKE_PERF=$candidate" >> "$GITHUB_ENV"
    break
  fi
done
