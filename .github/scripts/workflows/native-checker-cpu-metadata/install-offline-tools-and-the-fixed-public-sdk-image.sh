#!/usr/bin/env bash
set -euo pipefail
[[ "$CAPTURE_RUN" =~ ^[0-9]+$ && "$CAPTURE_ARTIFACT" =~ ^[0-9]+$ ]]
sudo apt-get update -qq
sudo apt-get install -y linux-tools-generic binutils
shopt -s nullglob
for candidate in /usr/lib/linux-tools/*/perf /usr/lib/linux-tools-*/perf; do
  if [[ -x "$candidate" ]]; then
    echo "NATIVE_PERF=$candidate" >> "$GITHUB_ENV"
    break
  fi
done
provider="$RUNNER_TEMP/checker-sdk"
mkdir "$provider"
node "$VIZE_CI_FILES/install-offline-tools-and-the-fixed-public-sdk-image.cjs" > "$provider/package.json"
npm install --prefix "$provider" --ignore-scripts --no-audit --no-fund
