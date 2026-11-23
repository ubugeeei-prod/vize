#!/usr/bin/env bash
set -euo pipefail
provider="$RUNNER_TEMP/mq-content-mapper-host"
proof="$RUNNER_TEMP/mq-content-mapper-host-proof"
env -u GH_TOKEN -u GITHUB_TOKEN -u NODE_AUTH_TOKEN -u NPM_TOKEN -u NODE_OPTIONS -u NODE_PATH \
  node tools/support/release/type-admission-mapper-host.ts "$provider" "$proof"
runtime="$provider/node_modules/@typescript/typescript-linux-x64/lib/tsc"
test -x "$runtime"
# Projection parity, memo and adjacent-attribute cases use this qualified published beta.
# Other conformance roles keep their own checker and companion closure.
printf 'VIZE_TEST_PROJECTION_CONTENT_MAPPER_TSGO=%s\n' "$runtime" >> "$GITHUB_ENV"
printf 'VIZE_TEST_MEMO_CONTENT_MAPPER_TSGO=%s\n' "$runtime" >> "$GITHUB_ENV"
printf 'VIZE_TEST_RECOVERED_ATTRIBUTE_CONTENT_MAPPER_TSGO=%s\n' "$runtime" >> "$GITHUB_ENV"
