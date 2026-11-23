#!/usr/bin/env bash
set -euo pipefail
provider="$RUNNER_TEMP/original27-current-provider"
mkdir "$provider"
cp original27-fixtures/tests/_fixtures/differential/typechecker/constructor-overload-11023-original27/provider-package.json "$provider/package.json"
npm install --prefix "$provider" --ignore-scripts --include=optional --no-audit --no-fund --registry=https://registry.npmjs.org > "$RUNNER_TEMP/original27-current-evidence/provider-install.stdout" 2> "$RUNNER_TEMP/original27-current-evidence/provider-install.stderr"
