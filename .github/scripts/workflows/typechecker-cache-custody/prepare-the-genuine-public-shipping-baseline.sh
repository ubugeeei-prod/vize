#!/usr/bin/env bash
set -euo pipefail
mkdir "$RUNNER_TEMP/cache-provider" "$RUNNER_TEMP/cache-public" "$RUNNER_TEMP/cache-evidence"
cd "$RUNNER_TEMP/cache-provider"
cat > package.json <<'JSON'
{"private":true,"dependencies":{"vize":"0.602.0","@vizejs/native":"0.602.0","vue":"3.5.44","typescript":"6.0.3","typescript-native":"npm:typescript@7.1.0-beta","vue-tsc":"3.3.13"}}
JSON
npm install --ignore-scripts --no-audit --no-fund
cd "$RUNNER_TEMP/cache-public"
curl --fail --location --retry 3 --output vize.tar.gz https://github.com/ubugeeei-prod/vize/releases/download/v0.602.0/vize-x86_64-unknown-linux-gnu.tar.gz
printf '%s\n' '4a75062b16eeee03449c0e471f3776be383f16e99e0820fe6a170b40e63ff93e  vize.tar.gz' | sha256sum --check -
tar -xzf vize.tar.gz
test -x vize
