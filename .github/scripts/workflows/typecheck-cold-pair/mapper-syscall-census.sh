#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/mapper-syscall-census.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
sudo apt-get install -y --no-install-recommends strace > /dev/null
size=$(echo "$SIZES" | tr "," "\n" | sort -n | head -1)
node tools/benchmarks/scripts/content-mapper-census.ts \
  --bin base=pair-bin/base/vize --bin head=pair-bin/head/vize \
  --project "$RUNNER_TEMP/pair/vue-$size" --stream "$RUNNER_TEMP/pair/stream-$size.bin" \
  --label "linux-ext4, $size SFCs" --out "$RUNNER_TEMP/pair-out/census.md" > /dev/null
cat "$RUNNER_TEMP/pair-out/census.md" >> "$GITHUB_STEP_SUMMARY"
