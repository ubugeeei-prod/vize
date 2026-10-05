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
