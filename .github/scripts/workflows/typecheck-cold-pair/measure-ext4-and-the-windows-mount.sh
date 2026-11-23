#!/usr/bin/env bash
set -euo pipefail
[[ "$PROJECTS" =~ ^(vue|monorepo):[0-9]+(,(vue|monorepo):[0-9]+)*$ ]] && [[ "$RUNS" =~ ^[1-9][0-9]?$ ]]
[[ "$RUNTIME" =~ ^(tsgo|tsrs)$ ]] && [[ "$CHECKERS" =~ ^(1|8)$ ]]
# Binaries and the cache root stay on ext4 in both cells; only the project location differs.
bin="$HOME/pair-bin"
mkdir -p "$bin/base" "$bin/head" "$WSL_MOUNT_ROOT"
cp pair-bin/base/vize "$bin/base/vize" && cp pair-bin/head/vize "$bin/head/vize"
chmod +x "$bin"/*/vize
summary_path=$(wslpath "$GITHUB_STEP_SUMMARY")
for entry in "wsl2-ext4=$HOME/vize-cold-pair" "wsl2-windows-mount=$WSL_MOUNT_ROOT/projects"; do
  label=${entry%%=*}
  node tools/benchmarks/scripts/typecheck-cold-pair.ts \
    --bin "base=$bin/base/vize" --bin "head=$bin/head/vize" \
    --root "${entry#*=}" --cache-root "$HOME/vize-cold-pair-cache" \
    --projects "$PROJECTS" --runs "$RUNS" --runtime "$RUNTIME" --checkers "$CHECKERS" $ALLOW \
    --label "$label-$RUNTIME-c$CHECKERS" --out "$WSL_MOUNT_ROOT/out" > /dev/null
  cat "$WSL_MOUNT_ROOT/out/cold-pair-$label-$RUNTIME-c$CHECKERS.md" >> "$summary_path"
done
