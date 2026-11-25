#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-template/scripts/verify-comment-root-compiler-modules-in-the-actual-vue-runtime.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize_atelier_dom --profile ci --test integration dev_root_fragment::
NODE_ENV=development vp node --test tests/tooling/dev-root-fragment-runtime.test.ts
NODE_ENV=production vp node --test tests/tooling/dev-root-fragment-runtime.test.ts
