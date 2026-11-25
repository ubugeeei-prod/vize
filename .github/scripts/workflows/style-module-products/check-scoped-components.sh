#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/style-module-products/check-scoped-components.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
export NODE_PATH="$RUNNER_TEMP/scoped-oracle/node_modules"
version=$(node "$VIZE_CI_FILES/read-pinned-sass-version.ts")
npm install --prefix "$RUNNER_TEMP/scoped-oracle" --no-audit --no-fund --ignore-scripts --save-exact "sass@$version" @vue/compiler-sfc@3.5.35 @jridgewell/sourcemap-codec@1.6.0 postcss-selector-parser@7.1.6 vue@3.5.35
export VIZE_SCSS_MANIFEST="$RUNNER_TEMP/scoped-components"
export VIZE_REQUIRE_REPOSITORY_SCSS=1
git ls-files -z -- '*.vue' '*.vue.txt' '*.scss' > "$VIZE_SCSS_MANIFEST"
flags=(--locked --profile ci-opt)
cargo test "${flags[@]}" -p vize_atelier_sfc --features native-scoped-scss --test native_sfc_scss observed_scoped_components_match_published_style_and_render_products -- --ignored
export VIZE_TOOLCHAIN_HISTORY="$RUNNER_TEMP/scoped-revisions"
export VIZE_STYLED_SESSION_OUTPUT="$RUNNER_TEMP/styled-session.json"
export VIZE_SCOPED_SESSION_OUTPUT="$RUNNER_TEMP/scoped-session.json"
features=experimental-lexical-arrow-queries,experimental-recovered-literal-prefix,experimental-scoped-component-session,experimental-style-lint-products,micro-profiling
cargo test "${flags[@]}" -p vize_maestro --features "$features" --lib toolchain_session_from_one_open_buffer
node crates/vize_atelier_sfc/tests/component_scss/oracle.ts < "$VIZE_STYLED_SESSION_OUTPUT"
node crates/vize_atelier_sfc/tests/component_scss/oracle.ts < "$VIZE_SCOPED_SESSION_OUTPUT"
node "$VIZE_CI_FILES/check-style-revisions.ts"
cargo test "${flags[@]}" -p vize_atelier_sfc --features bundler-block-products --test bundler_toolchain_session -- --ignored
cargo test "${flags[@]}" --manifest-path crates/vize_incremental/providers/graph/Cargo.toml --test incremental original_template_graphs_follow_the_same_toolchain_session -- --ignored
