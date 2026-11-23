#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/js-native-fault-evidence-orchestration"
cp .github/actions/test-js-packages-with-history/native-fault-evidence.ts "$RUNNER_TEMP/native-fault-evidence.ts"
cp .github/actions/test-js-packages-with-history/native-fault-core.ts "$RUNNER_TEMP/native-fault-core.ts"
cp .github/actions/test-js-packages-with-history/native-fault-support.ts "$RUNNER_TEMP/native-fault-support.ts"
cp .github/actions/test-js-packages-with-history/native-fault-retention.ts "$RUNNER_TEMP/native-fault-retention.ts"
cp .github/actions/test-js-packages-with-history/native-fault-resources.ts "$RUNNER_TEMP/native-fault-resources.ts"
node "$VIZE_CI_FILES/retain-diagnostic-orchestration-before-checking-out-the-failed-source.ts"
