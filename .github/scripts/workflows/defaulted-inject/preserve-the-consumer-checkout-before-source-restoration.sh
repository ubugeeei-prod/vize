#!/usr/bin/env bash
mkdir -p "$RUNNER_TEMP/defaulted-inject-proof"
git rev-parse HEAD > "$RUNNER_TEMP/defaulted-inject-proof/consumer-checkout.txt"
