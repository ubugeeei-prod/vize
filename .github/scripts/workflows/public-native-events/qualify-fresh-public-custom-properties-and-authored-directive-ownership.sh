#!/usr/bin/env bash
node tools/support/release/public_custom_properties/verify.ts \
  "$PUBLIC_VERSION" "$RUNNER_TEMP/public-custom-properties-consumer" \
  "$RUNNER_TEMP/cpp-release-identity.json"
