#!/usr/bin/env bash
# Required source-built public CLI and editor controls for original #7874.
set -euo pipefail
[[ "${VIZE_TEST_REQUIRE_TSGO:-}" == 1 ]]
[[ -n "${SOURCE_SHA:-}" && -n "${RUNNER_TEMP:-}" ]]
unset VIZE_TEST_DISABLE_TSGO
export VIZE_UNKNOWN_TEMPLATE_CAPTURE="$RUNNER_TEMP/unknown-template-options"
options=(--locked --profile ci-opt --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16')
cargo test "${options[@]}" -p vize --test check_unknown_template_options_cli --config 'profile.ci-opt.package.vize.strip="symbols"' -- --nocapture
cargo test "${options[@]}" -p vize_maestro --lib editor_typecheck_unknown_options_tests -- --nocapture

# Execute the unchanged full compatibility cases that failed protected delivery.
export VIZE_UNKNOWN_COMPAT_CAPTURE="$VIZE_UNKNOWN_TEMPLATE_CAPTURE/compatibility"
export CORSA_PATH="$VIZE_TEST_TSGO_PATH"
cargo test "${options[@]}" -p vize --test check_canon_fallthrough_attrs_cli --config 'profile.ci-opt.package.vize.strip="symbols"' -- --nocapture
cargo test "${options[@]}" -p vize_canon --lib compatibility_capture:: -- --nocapture
cargo test "${options[@]}" -p vize_canon --lib fallthrough_unknown_attrs:: -- --nocapture
cargo test "${options[@]}" -p vize_canon --lib wide_props_type_complexity:: -- --nocapture
cargo test "${options[@]}" -p vize_canon --lib generic_props::declaration_emit:: -- --nocapture
