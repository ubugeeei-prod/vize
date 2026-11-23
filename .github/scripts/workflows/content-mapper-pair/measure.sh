#!/usr/bin/env bash
set -euo pipefail
[[ "$SIZES" =~ ^[0-9]+(,[0-9]+)*$ ]] && [[ "$RUNS" =~ ^[1-9][0-9]?$ ]]
if ! /usr/bin/time --version 2>&1 | grep -qi 'GNU time'; then
  sudo apt-get update -qq && sudo apt-get install -y --no-install-recommends time
fi
chmod +x pair-bin/*/vize
reference=()
if [[ "$REFERENCE" == "true" ]]; then
  npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund > /dev/null
  hosts=tools/benchmarks/typecheck-hosts/node_modules
  reference=(--reference "vue-mapper=$hosts/@vue/content-mapper/server.js" --typescript "$hosts/typescript")
fi
node tools/benchmarks/scripts/content-mapper-pair.ts \
  --bin base=pair-bin/base/vize --bin head=pair-bin/head/vize "${reference[@]}" \
  --root "$RUNNER_TEMP/pair" --sizes "$SIZES" --runs "$RUNS" $ALLOW \
  --label linux-ext4 --out "$RUNNER_TEMP/pair-out" > /dev/null
cat "$RUNNER_TEMP/pair-out/pair-linux-ext4.md" >> "$GITHUB_STEP_SUMMARY"
