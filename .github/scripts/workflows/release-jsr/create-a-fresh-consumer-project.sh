#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-jsr/create-a-fresh-consumer-project.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
mkdir "$RUNNER_TEMP/consumer"
cp tools/support/release/jsr/consumer.ts "$RUNNER_TEMP/consumer/consumer.ts"
echo '{ "name": "vize-jsr-consumer", "private": true, "type": "module" }' \
  > "$RUNNER_TEMP/consumer/package.json"
