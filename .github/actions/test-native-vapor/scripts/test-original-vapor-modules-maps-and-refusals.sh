#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-vapor/scripts/test-original-vapor-modules-maps-and-refusals.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_l4 --test native_vapor
test -s "$VIZE_NATIVE_VAPOR_CAPTURE"
vp node --test tests/tooling/native-vapor-reference.test.mjs
test -s "$VIZE_NATIVE_VAPOR_RUNTIME_CAPTURE"
test -s "$VIZE_NATIVE_VAPOR_CAPTURE.class-cases.json"
vp node --test tests/tooling/native-vapor-static-class-reference.test.mjs
test -s "$VIZE_NATIVE_VAPOR_CLASS_RUNTIME_CAPTURE.development.stdout"
test -s "$VIZE_NATIVE_VAPOR_CLASS_RUNTIME_CAPTURE.production.stdout"
cargo test --locked --profile ci -p vize_atelier_sfc --test native_vapor_sfc
test -s "$VIZE_NATIVE_VAPOR_SFC_CAPTURE"
vp node --test tests/tooling/native-vapor-sfc-reference.test.mjs
test -s "$VIZE_NATIVE_VAPOR_SFC_RUNTIME_CAPTURE"
test -s "$VIZE_NATIVE_VAPOR_PRIMARY_LIFECYCLE_CAPTURE"
test -s "$VIZE_NATIVE_VAPOR_PROCESS_CAPTURE"
cargo test --locked --profile ci -p vize_atelier_sfc --test native_vapor_setup_sfc
test -s "$VIZE_NATIVE_VAPOR_SETUP_SFC_CAPTURE"
vp node --test tests/tooling/native-vapor-setup-sfc-reference.test.mjs
test -s "$VIZE_NATIVE_VAPOR_SETUP_SFC_RUNTIME_CAPTURE"
cargo test --locked --profile ci -p vize_atelier_jsx --test v_slots_forwarding
test -s "$VIZE_JSX_VAPOR_SLOTS_CAPTURE"
vp node --test tests/tooling/jsx-runtime-slots-vapor.test.mjs
test -s "$VIZE_JSX_VAPOR_SLOTS_CAPTURE.runtime.json"
