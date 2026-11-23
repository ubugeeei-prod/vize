#!/usr/bin/env bash
set -euo pipefail
within_budget() {
  local remaining=$((TEMPLATE_ALIAS_DEADLINE_MS - $(date +%s%3N)))
  test "$remaining" -gt 0
  local seconds=$(((remaining + 999) / 1000))
  if [ "$seconds" -gt 300 ]; then seconds=300; fi
  timeout --signal=TERM --kill-after=5s "${seconds}s" "$@"
}
within_budget vp install --frozen-lockfile --filter . --filter ./npm/cli... > "$RUNNER_TEMP/mapper-public-stages/driver-install.log" 2>&1
within_budget node tools/benchmarks/scripts/template-alias-paired-11822-contracts.ts > "$RUNNER_TEMP/mapper-public-stages/contracts.log" 2>&1
mkdir "$RUNNER_TEMP/mapper-public-provider"
cp tests/_fixtures/tooling/public-mapper-provider/package.json.txt "$RUNNER_TEMP/mapper-public-provider/package.json"
cp tests/_fixtures/tooling/public-mapper-provider/package-lock.json.txt "$RUNNER_TEMP/mapper-public-provider/package-lock.json"
(cd "$RUNNER_TEMP/mapper-public-provider" && within_budget npm ci --ignore-scripts --no-audit --no-fund) > "$RUNNER_TEMP/mapper-public-stages/provider-install.log" 2>&1
