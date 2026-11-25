#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/09-eslint-plugin/scripts/verify-no-physical-sfc-reads-including-native-code.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if ! command -v strace >/dev/null; then sudo apt-get update; sudo apt-get install -y strace; fi
export VIZE_ESLINT_WATCHED_SOURCE="$RUNNER_TEMP/eslint-source-$SOURCE_SHA/Disk.vue"
strace -f -yy -e trace=open,openat,openat2 -o "$VIZE_ESLINT_EVIDENCE/source-opens.raw.log" node npm/linter/eslint/test/source-read-probe.mjs "$VIZE_ESLINT_WATCHED_SOURCE" > "$VIZE_ESLINT_EVIDENCE/source-read-packets.json"
node "$VIZE_CI_FILES/verify-no-physical-sfc-reads-including-native-code.ts"
