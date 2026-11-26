#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/09-n8n-adoption/scripts/verify-retained-parameter-grammar-through-whole-source-cli-and-json-rpc.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_patina --test integration -- valid_v_slot_regression:: valid_v_slot_parameter_bindings::
vp node --test tests/tooling/patina-slot-parameter-bindings-oracle.test.ts
vp node tests/tooling/support/n8n-slot-parameters-acceptance.ts "target/n8n-cli-config/${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}/slot-parameters"
