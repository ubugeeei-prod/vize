#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vscode-startup-bench/measure-ext4-and-the-windows-mount.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$ROUNDS" =~ ^[1-9][0-9]?$ ]] && [[ "$SFCS" =~ ^[0-9]+$ ]] && [[ "$NOISE" =~ ^[0-9]+$ ]]
# Extensions and the server live on ext4, as a WSL remote installs them.
cp -R ext-builds "$HOME/ext-builds"
summary=$(wslpath "$GITHUB_STEP_SUMMARY")
mkdir -p "$WSL_MOUNT_ROOT/out"
for layout in $LAYOUTS; do
  [[ "$layout" =~ ^(root|nested)$ ]]
  for entry in "wsl2-ext4=$HOME/ws-$layout" "wsl2-windows-mount=$WSL_MOUNT_ROOT/ws-$layout"; do
    label=${entry%%=*}
    node editors/vscode/test/run-startup-latency.ts generate --root "${entry#*=}" \
      --layout "$layout" --sfcs "$SFCS" --noise "$NOISE"
    xvfb-run -a node editors/vscode/test/run-startup-latency.ts run --code "$(cat "$HOME/code-path")" \
      --server "$HOME/server/vize" --workspace "${entry#*=}" \
      --extension "base=$HOME/ext-builds/base" --extension "head=$HOME/ext-builds/head" \
      --rounds "$ROUNDS" --profiles "$HOME/profiles" --out "$WSL_MOUNT_ROOT/out/$label-$layout" \
      --title "WSL2 $label, $layout layout, $SFCS SFCs + $NOISE files" >> "$summary"
  done
done
