#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typechecker-native-phases/qualify-original-outside-import-types-with-the-source-cli-and-official-n.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
git show "$GITHUB_WORKFLOW_SHA:tools/benchmarks/scripts/typechecker-native-project-retirement.mjs" > "$RUNNER_TEMP/typechecker-native-project-retirement.mjs"
cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo test --locked --profile ci-opt -p vize --test integration --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' --config 'profile.ci-opt.package.vize.strip="symbols"' -- check_outside_import_types_cli:: --nocapture
export VIZE_TEST_TSGO_PATH="$(node "$VIZE_CI_FILES/qualify-original-outside-import-types-with-the-source-cli-and-official-n.ts")"
export CORSA_PATH="$VIZE_TEST_TSGO_PATH"
cargo test --locked --profile ci-opt -p vize_canon --test original_program_check process_reuse:: --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture || exit $?; cargo test --locked --profile ci-opt -p vize_canon --lib lsp_client::editor_lsp::original_program::process_tests:: --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture || exit $?; cargo test --locked --profile ci-opt -p vize_canon --lib original_outside_import_preserves_types_in_native_editor --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture
cargo test --locked --profile ci-opt -p vize_canon --lib native_editor_resolves_package_alias_from_monorepo_root --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture
node "$VIZE_CI_FILES/qualify-original-outside-import-types-with-the-source-cli-and-official-n-2.ts"
