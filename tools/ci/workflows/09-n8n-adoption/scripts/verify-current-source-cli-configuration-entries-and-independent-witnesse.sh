#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/09-n8n-adoption/scripts/verify-current-source-cli-configuration-entries-and-independent-witnesse.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
vp node --test tests/tooling/n8n-cli-config-contract.test.mjs tests/tooling/n8n-cli-config-failure-custody.test.mjs
vp node tests/tooling/support/n8n-cli-config-acceptance.mjs \
  "target/n8n-cli-config/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"
vp node tests/tooling/support/n8n-cli-config-semantic-acceptance.mjs \
  "target/n8n-cli-config/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}/semantic-routing"
