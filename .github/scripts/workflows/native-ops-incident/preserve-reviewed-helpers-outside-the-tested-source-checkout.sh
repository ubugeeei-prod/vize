#!/usr/bin/env bash
mkdir "$RUNNER_TEMP/native-ops-harness"
cp .github/scripts/native-ops-driver.ts .github/scripts/native-ops-data.ts "$RUNNER_TEMP/native-ops-harness/"
sha256sum "$RUNNER_TEMP/native-ops-harness/"*.ts
