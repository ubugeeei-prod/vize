#!/usr/bin/env bash
node tests/tooling/support/jsx-child-profile-contract.ts verify "$RUNNER_TEMP/jsx-authored-native"
node "$VIZE_CI_FILES/compare-jsx-source-contracts.ts"
node tests/tooling/support/jsx-child-profile-contract.ts report "$RUNNER_TEMP/jsx-authored-native"
