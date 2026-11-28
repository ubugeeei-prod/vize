#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-imported-props/scripts/prepare-cpu-sampling.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends linux-tools-generic
sudo sysctl -w kernel.perf_event_paranoid=-1 kernel.kptr_restrict=0
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [ -x "$candidate" ]; then
    echo "IMPORTED_PROPS_PERF=$candidate" >> "$GITHUB_ENV"
    exit 0
  fi
done
exit 1
