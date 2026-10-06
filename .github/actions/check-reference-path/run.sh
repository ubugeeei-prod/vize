#!/usr/bin/env bash
set +e
cargo test --locked --profile ci-opt -p vize --test check_reference_path_cli
cli_exit=$?
cargo test --locked --profile ci-opt -p vize_canon --lib reference_paths::
native_exit=$?
cargo test --locked --profile ci-opt -p vize_canon --lib css_side_effect_import::
assets_exit=$?
[[ "$cli_exit" == 0 && "$native_exit" == 0 && "$assets_exit" == 0 ]] && VIZE_NESTED_BATCH_CAPTURE_DIR="$RUNNER_TEMP/outside-import-types/nested-batch-config" bash .github/actions/check-batch-generated-config/run.sh &&
  bash .github/actions/check-computed-inlay/run.sh && \
  bash .github/actions/check-editor-jsconfig/run.sh
