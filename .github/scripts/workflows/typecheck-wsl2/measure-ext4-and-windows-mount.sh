#!/usr/bin/env bash
set -euo pipefail
[[ "$SIZES" =~ ^([0-9]+(,[0-9]+)*)?$ ]]
[[ "$GIANT_BLOCKS" =~ ^[0-9]+$ ]]
[[ "$MONOREPO_SFCS" =~ ^[0-9]+$ ]]
[[ "$RUNS" =~ ^[1-9][0-9]?$ ]]
ext4="$HOME/vize-wsl2"
mounted="$WSL_MOUNT_ROOT/mounted"
output="$WSL_MOUNT_ROOT/results"
mkdir -p "$ext4" "$mounted" "$output"
node tools/benchmarks/scripts/typecheck-wsl2.ts \
  --location "ext4=$ext4" --location "windows-mount=$mounted" \
  --vize "$VIZE_SPEC" --sizes "$SIZES" --giant-blocks "$GIANT_BLOCKS" \
  --monorepo-sfcs "$MONOREPO_SFCS" --runs "$RUNS" --out "$output"
summary_path=$(wslpath "$GITHUB_STEP_SUMMARY")
cat "$output/summary.md" >> "$summary_path"
wsl.exe --list --verbose
