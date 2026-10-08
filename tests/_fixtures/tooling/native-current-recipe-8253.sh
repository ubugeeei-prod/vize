set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cd "$NATIVE_PHASE_SOURCE_ROOT"
VIZE_UNCHECKED_INDEX_CAPTURE="$RUNNER_TEMP/unchecked-index-access" cargo test --locked --profile ci-opt -p vize --test check_template_emit_cli --test check_vue_helper_scope_cli --test lint_unchecked_indexed_access_cli --test check_canon_define_model_modifiers_cli --test lsp_generic_prop_hover_cli --test lsp_data_aria_attributes_cli --test tsconfig_diamond --test check_tsconfig_types_extends_cli --test lsp_bare_script_symbols_cli --test check_tsconfig_bom_cli --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' --config 'profile.ci-opt.package.vize.strip="symbols"' -- --nocapture
export VIZE_TEST_TSGO_PATH="$(node --input-type=module -e 'import {readFileSync,readdirSync} from "node:fs";import {join} from "node:path";const root=process.env.VIZE_TEMPLATE_EMIT_CAPTURE;const name=readdirSync(root,{withFileTypes:true}).find(e=>e.isDirectory()).name;process.stdout.write(JSON.parse(readFileSync(join(root,name,"oracle.json"),"utf8")).nativeBinary)')"
cargo test --locked --profile ci-opt -p vize_maestro --lib editor_typecheck_emit_tests --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture
cargo test --locked --profile ci-opt -p vize_maestro --lib editor_typecheck_vue_helper_tests --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture
export VIZE_TEMPLATE_EMIT_TS40_CAPTURE="$RUNNER_TEMP/template-emit-ts40-projection"
cargo test --locked --profile ci-opt -p vize_maestro --test davinci_ts40_projection current_projection_matrix_is_exact_and_non_empty --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture
node tools/benchmarks/scripts/typechecker-template-emit-projection-receipt.mjs
node tools/benchmarks/scripts/typechecker-native-template-emit-receipt.mjs
node tools/benchmarks/scripts/typechecker-native-vue-helper-receipt.mjs
bash tools/benchmarks/scripts/typechecker-native-unknown-options.sh && cargo test --locked --profile ci-opt -p vize_patina --test documented_rule_examples documented_type_aware_examples_match_with_the_required_corsa_runtime --config 'profile.ci-opt.inherits="release"' --config 'profile.ci-opt.lto="thin"' --config 'profile.ci-opt.codegen-units=16' -- --nocapture
