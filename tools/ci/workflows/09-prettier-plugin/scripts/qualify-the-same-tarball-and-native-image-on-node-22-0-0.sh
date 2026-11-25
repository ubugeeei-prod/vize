#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/09-prettier-plugin/scripts/qualify-the-same-tarball-and-native-image-on-node-22-0-0.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node npm/native/scripts/test-preparation.mjs node npm/formatter/prettier/test/consumer.mjs \
  --mode source-native \
  --expected-node 22.0.0 \
  --tarball-directory "$RUNNER_TEMP/prettier-source-consumer" \
  --output "$RUNNER_TEMP/prettier-node22-consumer"
