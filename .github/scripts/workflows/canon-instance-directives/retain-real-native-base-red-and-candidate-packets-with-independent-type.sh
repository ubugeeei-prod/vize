#!/usr/bin/env bash
set -euo pipefail
for source in tools/benchmarks/scripts/canon-instance-directives*.ts; do
  node --check "$source"
done
node tools/benchmarks/scripts/canon-instance-directives.ts "$GITHUB_WORKSPACE" "$RUNNER_TEMP/instance-directive-provider" "$RUNNER_TEMP/instance-directives/binaries/base/vize" "$RUNNER_TEMP/instance-directives/binaries/head/vize" "$RUNNER_TEMP/instance-directives/packets"
