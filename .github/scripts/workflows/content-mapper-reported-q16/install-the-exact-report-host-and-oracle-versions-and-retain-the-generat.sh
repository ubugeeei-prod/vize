#!/usr/bin/env bash
set -uo pipefail
node --version > "$Q16_OUT/provider-node-version.txt"
npm --version > "$Q16_OUT/provider-npm-version.txt"
npm install --prefix "$Q16_PROVIDER" --package-lock-only --ignore-scripts --no-audit --no-fund --registry=https://registry.npmjs.org > "$Q16_OUT/provider-lock.stdout.bin" 2> "$Q16_OUT/provider-lock.stderr.bin"
lock_status=$?
printf '%s\n' "$lock_status" > "$Q16_OUT/provider-lock.status.txt"
test "$lock_status" = 0 || exit "$lock_status"
cp "$Q16_PROVIDER/package.json" "$Q16_OUT/provider-package.json"
cp "$Q16_PROVIDER/package-lock.json" "$Q16_OUT/provider-package-lock.json"
npm ci --prefix "$Q16_PROVIDER" --ignore-scripts --no-audit --no-fund --registry=https://registry.npmjs.org > "$Q16_OUT/provider-install.stdout.bin" 2> "$Q16_OUT/provider-install.stderr.bin"
install_status=$?
printf '%s\n' "$install_status" > "$Q16_OUT/provider-install.status.txt"
exit "$install_status"
