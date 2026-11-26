#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/09-prettier-plugin/scripts/install-the-tarball-in-a-fresh-consumer-and-exercise.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node npm/native/scripts/test-preparation.mjs node npm/formatter/prettier/test/consumer.mjs \
  --mode source-native \
  --expected-node 24.14.0 \
  --output "$RUNNER_TEMP/prettier-source-consumer"
