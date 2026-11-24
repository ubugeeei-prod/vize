#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/vscode-startup-bench/measure-2.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$ROUNDS" =~ ^[1-9][0-9]?$ ]] && [[ "$SFCS" =~ ^[0-9]+$ ]] && [[ "$NOISE" =~ ^[0-9]+$ ]]
temp=$(cygpath -m "$RUNNER_TEMP")
mkdir -p "$temp/server"
cp ext-builds/server/vize.exe "$temp/server/vize.exe"
"$temp/server/vize.exe" --version
VIZE_TEST_VSCODE_VERSION="$VSCODE_VERSION" vp run --workspace-root test:vscode-extension:download
code=$(cd editors/vscode && VIZE_TEST_VSCODE_VERSION="$VSCODE_VERSION" node test/download-vscode.mjs | sed -n -f "$VIZE_CI_FILES/verified-editor-path.sed" | tail -1 | tr -d '\r')
test -f "$code"
mkdir -p "$temp/out"
for layout in $LAYOUTS; do
  [[ "$layout" =~ ^(root|nested)$ ]]
  node editors/vscode/test/run-startup-latency.ts generate --root "$temp/ws-$layout" \
    --layout "$layout" --sfcs "$SFCS" --noise "$NOISE"
  node editors/vscode/test/run-startup-latency.ts run --code "$code" \
    --server "$temp/server/vize.exe" --workspace "$temp/ws-$layout" \
    --extension "base=$(cygpath -m "$PWD")/ext-builds/base" --extension "head=$(cygpath -m "$PWD")/ext-builds/head" \
    --rounds "$ROUNDS" --profiles "$temp/profiles" --out "$temp/out/windows-$layout" \
    --title "Windows NTFS, $layout layout, $SFCS SFCs + $NOISE files" >> "$GITHUB_STEP_SUMMARY"
done
