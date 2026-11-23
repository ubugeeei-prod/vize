#!/usr/bin/env bash
[[ "$PUBLIC_JSR_VERSION" =~ ^0\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]
mkdir -p "$RUNNER_TEMP/jsr-public-consumer"
cp tools/support/release/jsr/consumer.ts "$RUNNER_TEMP/jsr-public-consumer/consumer.ts"
cp tools/support/release/jsr/consumer-types.ts.fixture "$RUNNER_TEMP/jsr-public-consumer/consumer-types.ts"
cp tools/support/release/jsr/consumer-deno.json "$RUNNER_TEMP/jsr-public-consumer/deno.json"
node "$VIZE_CI_FILES/prepare-the-isolated-public-consumer.ts"
