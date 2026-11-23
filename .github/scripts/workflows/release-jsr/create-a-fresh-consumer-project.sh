#!/usr/bin/env bash
mkdir "$RUNNER_TEMP/consumer"
cp tools/support/release/jsr/consumer.ts "$RUNNER_TEMP/consumer/consumer.ts"
echo '{ "name": "vize-jsr-consumer", "private": true, "type": "module" }' \
  > "$RUNNER_TEMP/consumer/package.json"
