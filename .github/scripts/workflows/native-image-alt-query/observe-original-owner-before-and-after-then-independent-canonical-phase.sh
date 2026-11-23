#!/usr/bin/env bash
set -euo pipefail
capture="$RUNNER_TEMP/native-image-alt"
"$capture/base-observer" > "$capture/base.json" 2> "$capture/base.stderr"
"$capture/head-observer" > "$capture/head.json" 2> "$capture/head.stderr"
"$capture/canonical-observer" > "$capture/canonical.json" 2> "$capture/canonical.stderr"
