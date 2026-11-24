#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/app-e2e-row/scripts/provision-node-22-for-authored-nuxt-fixture-preparation.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
runtime=target/app-e2e-logs/nuxt-prepare-runtime
mkdir -p "$runtime"
node "$GITHUB_ACTION_PATH/scripts/print-node-runtime.cjs" > "$runtime/checker-before.json"
vp --version > "$runtime/vp-version.txt"
vp env install 22 > "$runtime/node22-install.log" 2>&1
vp env exec --node 22 node "$GITHUB_ACTION_PATH/scripts/write-node-runtime.cjs" "$runtime/node22.json" > "$runtime/node22-exec.log" 2>&1
prep_node22="$(node "$GITHUB_ACTION_PATH/scripts/read-node22-path.cjs" "$runtime/node22.json")"
test -x "$prep_node22"
sha256sum "$prep_node22" > "$runtime/node22.sha256"
node "$GITHUB_ACTION_PATH/scripts/print-node-runtime.cjs" > "$runtime/checker-after.json"
cmp "$runtime/checker-before.json" "$runtime/checker-after.json"
printf '%s\n' "VIZE_APP_CHECK_PREPARE_NODE22=$prep_node22" >> "$GITHUB_ENV"
