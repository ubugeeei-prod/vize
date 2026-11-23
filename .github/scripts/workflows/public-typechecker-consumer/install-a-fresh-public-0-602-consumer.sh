#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/public-provider"
cd "$RUNNER_TEMP/public-provider"
cat > package.json <<'JSON'
{"private":true,"dependencies":{"vize":"0.602.0","@vizejs/native":"0.602.0","vue":"3.5.44","typescript":"6.0.3","typescript-native":"npm:typescript@7.1.0-beta"}}
JSON
npm install --ignore-scripts --no-audit --no-fund
