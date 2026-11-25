#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-sfc-dom/scripts/qualify-fresh-completed-lower-file-static-class-modules-in-both-runtime.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_l4 --test native_dom_static_class -- --nocapture
NODE_ENV=development vp node --test tests/tooling/native-dom-static-class-reference.test.ts
NODE_ENV=production vp node --test tests/tooling/native-dom-static-class-reference.test.ts
test -s "$VIZE_L4_DOM_STATIC_CLASS_CAPTURE"
test -s "${VIZE_L4_DOM_STATIC_CLASS_RUNTIME_CAPTURE}.development.json"
test -s "${VIZE_L4_DOM_STATIC_CLASS_RUNTIME_CAPTURE}.production.json"
