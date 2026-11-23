#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$RUNNER_TEMP/narrowed-slot-consumer"
cd "$RUNNER_TEMP/narrowed-slot-consumer"
npm init -y
npm install --prefer-online --ignore-scripts --no-audit --no-fund --save-exact vize@0.569.0 typescript@7.1.0-beta vue@3.5.44
npm view typescript@7.1.0-beta gitHead dist.integrity --json > beta-metadata.json
