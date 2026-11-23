set -euo pipefail

consumer="$RUNNER_TEMP/component-input-consumer"
beta="$RUNNER_TEMP/component-input-beta"
mkdir -p "$consumer"
cd "$consumer"
npm init -y
npm install --prefer-online --ignore-scripts --no-audit --no-fund --save-exact \
  vize@0.566.0 typescript@7.1.0-dev.20261024.1 vue@3.5.42

mkdir -p "$beta"
cd "$beta"
npm init -y
npm install --prefer-online --ignore-scripts --no-audit --no-fund --save-exact \
  typescript@7.1.0-beta
npm view typescript@7.1.0-beta gitHead dist.integrity --json > provider-metadata.json
node "$GITHUB_WORKSPACE/.github/scripts/component-input-diagnostic-ownership/verify-provider-metadata.ts"
