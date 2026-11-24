#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/cold-typechecker-scale/install-fixed-comparison-providers.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir "$RUNNER_TEMP/provider"
cd "$RUNNER_TEMP/provider"
cat > package.json <<'JSON'
{"private":true,"dependencies":{"vize":"0.596.0","@vizejs/native":"0.596.0","vue":"3.6.0-rc.13","typescript":"6.0.3","typescript-native":"npm:typescript@7.1.0-beta","vue-tsc":"3.3.13"}}
JSON
npm install --ignore-scripts --no-audit --no-fund
mkdir -p "$RUNNER_TEMP/cold-scale"
cp package.json package-lock.json "$RUNNER_TEMP/cold-scale/"
