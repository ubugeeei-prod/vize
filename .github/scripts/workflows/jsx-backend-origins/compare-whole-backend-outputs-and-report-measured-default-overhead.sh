#!/usr/bin/env bash
node --test tests/tooling/jsx-child-backend-profile-contract.test.ts tests/tooling/jsx-current-backend-profile-contract.test.ts
node tests/tooling/support/jsx-current-backend-profile-contract.ts "$RUNNER_TEMP/jsx-backend-origins"
