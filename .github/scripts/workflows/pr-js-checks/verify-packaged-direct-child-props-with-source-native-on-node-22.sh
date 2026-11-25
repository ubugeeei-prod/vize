#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-js-checks/verify-packaged-direct-child-props-with-source-native-on-node-22.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
capture="$RUNNER_TEMP/direct-child-props-source-11809"
mkdir -p "$capture"
node npm/native/scripts/native-build-receipt.mjs >"$capture/producer.stdout" 2>"$capture/producer.stderr"
(cd npm/cli && vp pack) >"$capture/cli-pack.stdout" 2>"$capture/cli-pack.stderr"
node tools/support/compat/content-mapper/direct-child-props-11809.ts --capture "$capture" >"$capture/consumer.stdout" 2>"$capture/consumer.stderr"
