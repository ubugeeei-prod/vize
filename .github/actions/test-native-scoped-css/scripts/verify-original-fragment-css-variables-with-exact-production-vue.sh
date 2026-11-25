#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-scoped-css/scripts/verify-original-fragment-css-variables-with-exact-production-vue.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_atelier_sfc --test ssr_fragment_css_vars -- --nocapture
cargo test --locked --profile ci -p vize_atelier_ssr --features legacy-differential --test ssr_fragment_css_vars -- --nocapture
vp node --test tests/tooling/ssr-fragment-css-vars.test.ts
cargo test --locked --profile ci -p vize_atelier_ssr --features legacy-differential --test nested_branch_fragments -- --nocapture
vp node --test tests/tooling/ssr-nested-branch-fragments.test.mjs
cargo test --locked --profile ci -p vize_atelier_ssr --features legacy-differential --test scope_attr_forwarding -- --nocapture
vp node --test tests/tooling/ssr-component-scope-attrs.test.ts
