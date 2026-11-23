#!/usr/bin/env bash
set -euo pipefail
fixture=tests/_fixtures/differential/typechecker/zero-argument-slot-constructor-11013/real-project/provider
mkdir "$RUNNER_TEMP/slot-provider"
cp "$fixture/package.json" "$fixture/package-lock.json" "$RUNNER_TEMP/slot-provider/"
cd "$RUNNER_TEMP/slot-provider"
printf '%s  %s\n' \
  9ca03c3cb103e1ff8d0a8c633145a743a056f21c5a0be56fa633a19541ab5256 \
  package-lock.json | sha256sum --check --strict
npm ci --ignore-scripts --no-audit --no-fund
node "$VIZE_CI_FILES/verify-public-package-tree.ts"
