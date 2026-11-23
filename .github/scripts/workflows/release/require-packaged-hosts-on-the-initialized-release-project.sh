#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$VIZE_TS45_OUT"
node tests/editor-conformance/run.ts neovim --out "$VIZE_TS45_OUT"
node tests/editor-conformance/run.ts helix --out "$VIZE_TS45_OUT"
xvfb-run -a node tests/editor-conformance/run.ts vscode --out "$VIZE_TS45_OUT"
find "$VIZE_TS45_OUT" -type f -print | LC_ALL=C sort
find "$VIZE_TS45_EXISTING_WORKSPACE" -type l -print \
  > "${VIZE_STEPS_ARTIFACTS_OUTPUTS_EVIDENCE}/project-links-after-hosts.txt"
find "$VIZE_TS45_EXISTING_WORKSPACE" \
  -path "$VIZE_TS45_EXISTING_WORKSPACE/node_modules" -prune -o -type l -print \
  > "${VIZE_STEPS_ARTIFACTS_OUTPUTS_EVIDENCE}/project-links-after-hosts-outside-node-modules.txt"
test ! -s "${VIZE_STEPS_ARTIFACTS_OUTPUTS_EVIDENCE}/project-links-after-hosts-outside-node-modules.txt"
find "$VIZE_TS45_EXISTING_WORKSPACE/node_modules" -type l -print \
  > "${VIZE_STEPS_ARTIFACTS_OUTPUTS_EVIDENCE}/dependency-links-after-hosts.txt"
while IFS= read -r link; do
  target=$(readlink -f "$link")
  case "$target" in
    "$VIZE_TS45_EXISTING_WORKSPACE/node_modules"/*) ;;
    *)
      echo "installed dependency link escapes the fresh project after host execution: $link -> $target" >&2
      exit 1
      ;;
  esac
done < "${VIZE_STEPS_ARTIFACTS_OUTPUTS_EVIDENCE}/dependency-links-after-hosts.txt"
if grep -R -F -n "$GITHUB_WORKSPACE" "$VIZE_TS45_EXISTING_WORKSPACE" \
  > "${VIZE_STEPS_ARTIFACTS_OUTPUTS_EVIDENCE}/workspace-links-after-hosts.txt"; then
  echo "fresh release project refers to the source checkout after host execution" >&2
  exit 1
fi
