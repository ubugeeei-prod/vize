#!/usr/bin/env bash
node "$VIZE_CI_FILES/validate-frozen-historical-pair-before-checkout.ts"
test "$(git rev-parse HEAD)" = "$GITHUB_SHA"
