#!/usr/bin/env bash
vp node --test tests/tooling/n8n-cli-config-contract.test.mjs tests/tooling/n8n-cli-config-failure-custody.test.mjs
vp node tests/tooling/support/n8n-cli-config-acceptance.mjs \
  "target/n8n-cli-config/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"
vp node tests/tooling/support/n8n-cli-config-semantic-acceptance.mjs \
  "target/n8n-cli-config/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}/semantic-routing"
