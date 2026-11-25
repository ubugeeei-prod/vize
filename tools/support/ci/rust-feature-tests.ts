// Tests and Clippy runs for experimental Cargo features that the workspace
// suites never enable. Each row builds one feature surface; the PR planner
// selects the rows whose crates a change affects, and full qualification runs
// every row.
//
// Commands are Cargo arguments after the subcommand. The runner adds
// `--locked` and `--profile ci` (unless a command names its own profile) and
// `-- -D warnings` to Clippy. Ignored tests stay out of the lane: they read
// corpora selected outside the repository or measure time and allocations.

export interface RustFeatureRow {
  readonly id: string;
  /** Workspace crates whose changes select the row. */
  readonly packages: readonly string[];
  readonly commands: readonly string[];
  readonly env?: Readonly<Record<string, string>>;
  /** Runs on Windows instead of Linux. */
  readonly windows?: boolean;
  /** Provisions the locked TypeScript runtime that native project tests require. */
  readonly runtime?: boolean;
}

const largeStack = { RUST_MIN_STACK: "16777216" };
const tsgo = {
  ...largeStack,
  VIZE_TEST_REQUIRE_TSGO: "1",
  VIZE_TEST_TSGO_PATH: "node_modules/@typescript/typescript-linux-x64/lib/tsc",
};

const listenerFeatures =
  "component-listener-queries,template-operand-queries,vize_l2/position-index";
const propFeatures =
  "authored-bind-shorthand,component-prop-queries,template-operand-queries,vize_l2/expression-queries,vize_l2/position-index";
const arrowFeatures = "lexical-arrow-initializers,template-operand-queries,vize_l2/position-index";
const importFeatures = "template-operand-queries,setup-component-imports,vize_l2/position-index";

export const rustFeatureRows: readonly RustFeatureRow[] = [
  {
    id: "art-sources",
    packages: ["vize_musea"],
    env: largeStack,
    commands: [
      "clippy -p vize_musea --features art-source-products --lib --no-deps",
      "test -p vize_musea --features art-source-products --test art_source_products",
    ],
  },
  {
    id: "bundler-blocks",
    packages: ["vize_atelier_sfc"],
    commands: [
      "clippy -p vize_atelier_sfc --features bundler-block-products --lib --no-deps",
      "test -p vize_atelier_sfc --features bundler-block-products --lib bundler::tests",
      "test -p vize_atelier_sfc --features bundler-block-products --test bundler_block_products -- --test-threads=1",
    ],
  },
  {
    id: "cli-feature-sets",
    packages: ["vize"],
    commands: [
      "check -p vize --bin vize --no-default-features",
      "check -p vize --bin vize --features glyph",
      "check -p vize --bin vize --features profiling",
    ],
  },
  {
    id: "client-server-decisions",
    packages: ["vize_l3", "vize_l4", "vize_atelier_sfc"],
    env: largeStack,
    commands: [
      "clippy -p vize_l3 --features client-server-decisions --lib --no-deps",
      "clippy -p vize_l4 --features client-server-decisions --lib --no-deps",
      "test -p vize_l4 --features client-server-decisions --test native_dom_file --test native_ssr --test native_selected_ssr --test native_selected_root_text --test native_selected_template_output",
      "clippy -p vize_atelier_sfc --features vize_l4/client-server-decisions --test client_server_decisions --no-deps",
      "test -p vize_atelier_sfc --features vize_l4/client-server-decisions --test client_server_decisions -- --test-threads=1",
    ],
  },
  {
    id: "component-listeners",
    packages: ["vize_l2", "vize_l1_to_l2", "vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_l2 --features component-listener-queries,template-operand-queries --lib --no-deps",
      `clippy -p vize_l1_to_l2 --features ${listenerFeatures} --lib --no-deps`,
      "clippy -p vize_maestro --features experimental-component-listeners,experimental-template-operand-products,micro-profiling --lib --no-deps",
      `test -p vize_l1_to_l2 --features ${listenerFeatures} --test expression_queries component_listeners:: -- --test-threads=1`,
      "test -p vize_maestro --features experimental-component-listeners,experimental-template-operand-products,micro-profiling --lib event_rename:: -- --test-threads=1",
      "test -p vize_maestro --features experimental-component-listeners,experimental-template-operand-products,micro-profiling --lib named_dom_event_expressions_keep_their_checker_links_and_editor_exports",
    ],
  },
  {
    id: "component-props",
    packages: ["vize_l2", "vize_l1_to_l2", "vize_maestro"],
    env: largeStack,
    commands: [
      `clippy -p vize_l1_to_l2 --features ${propFeatures} --lib --no-deps`,
      "clippy -p vize_maestro --features experimental-component-props,micro-profiling --lib --no-deps",
      "test -p vize_l2 --features component-prop-queries --lib original_operand_checks",
      `test -p vize_l1_to_l2 --features ${propFeatures} --test expression_queries shorthand:: -- --test-threads=1`,
      `test -p vize_l1_to_l2 --features ${propFeatures} --test expression_query_alloc`,
      `test -p vize_l1_to_l2 --features ${propFeatures} --test expression_queries component_props:: -- --test-threads=1`,
      "test -p vize_maestro --features experimental-component-props,micro-profiling --lib component_props::original:: -- --test-threads=1",
    ],
  },
  {
    id: "declaration-products",
    packages: ["vize_l1", "vize_canon", "vize_maestro"],
    env: tsgo,
    commands: [
      "clippy -p vize_canon --features ambient-project-declarations --lib --no-deps",
      "clippy -p vize_maestro --features experimental-ambient-project --lib --no-deps",
      "test -p vize_canon --features ambient-project-declarations --lib virtual_project::ambient_declarations::",
      "test -p vize_canon --features ambient-project-declarations --lib tracked_documents::tests::",
      "test -p vize_maestro --features experimental-ambient-project --lib corsa::ambient_project:: -- --test-threads=1",
      "clippy -p vize_canon --lib --features plain-script-bridges --no-deps",
      "test -p vize_canon --lib --features plain-script-bridges plain_exports::observed:: -- --test-threads=1",
      "clippy -p vize_l1 --features authored-declarations --lib --no-deps",
      "clippy -p vize_canon --features authored-declarations --lib --no-deps",
      "clippy -p vize_maestro --features experimental-declaration-products,micro-profiling --lib --no-deps",
      "test -p vize_canon --features authored-declarations --lib batch::vue_template_ref_module::",
      "test -p vize_canon --features authored-declarations --lib batch::import_rewriter_dts_tests::",
      "test -p vize_maestro --features experimental-declaration-products,micro-profiling --lib server::state::global_tag_names::",
    ],
  },
  {
    id: "diagnostic-positions",
    packages: ["vize_l0", "vize_l4"],
    commands: [
      "test -p vize_l0 --features diagnostic-positions --lib line_index",
      "clippy -p vize_l0 --features diagnostic-positions --lib --no-deps",
      "clippy -p vize_l4 --features diagnostic-positions --lib --test program_projection --no-deps",
      "test -p vize_l4 --features diagnostic-positions --test program_projection --test vue_projection --test jsx_module_projection",
    ],
  },
  {
    id: "editor-script-queries",
    packages: ["vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_maestro --features experimental-script-queries,micro-profiling --lib --no-deps",
      "test -p vize_maestro --features experimental-script-queries,micro-profiling --lib server::document_structure::symbols::tests",
      "test -p vize_maestro --features experimental-script-queries,micro-profiling --lib ide::file_rename::",
      "test -p vize_maestro --features experimental-script-queries,micro-profiling --lib source_project::script_queries:: -- --test-threads=1",
      "clippy -p vize_maestro --features experimental-script-tools,micro-profiling --lib --no-deps",
      "test -p vize_maestro --features experimental-script-tools,micro-profiling --lib source_project::script_queries::tools:: -- --test-threads=1",
      "clippy -p vize_maestro --features experimental-checker-script-queries,micro-profiling --lib --no-deps",
      "test -p vize_maestro --features experimental-checker-script-queries,micro-profiling --lib source_project::script_queries:: -- --test-threads=1",
    ],
  },
  {
    id: "expression-queries",
    packages: ["vize_l2", "vize_l1_to_l2", "vize_croquis_cf"],
    env: largeStack,
    commands: [
      "clippy -p vize_l2 -p vize_l1_to_l2 --features vize_l1_to_l2/setup-component-imports --lib --no-deps",
      `test -p vize_l1_to_l2 --features ${importFeatures} --test expression_queries --test expression_query_alloc`,
      "clippy -p vize_croquis_cf --features setup-render-products --lib --no-deps",
      "test -p vize_croquis_cf --features setup-render-products --lib facts::",
    ],
  },
  {
    id: "graph-queries",
    packages: ["vize_l2", "vize_l2_to_l3", "vize_l3", "vize_incremental"],
    env: largeStack,
    commands: [
      "test -p vize_l2_to_l3 --features graph-queries",
      "test -p vize_l2 --features position-index --lib query::position::index::",
      "test -p vize_incremental --features resident-providers",
      "test --profile ci-opt --manifest-path crates/vize_incremental/providers/graph/Cargo.toml",
      "clippy -p vize_l3 -p vize_incremental --features vize_incremental/resident-providers,vize_l3/graph-queries --lib --no-deps",
    ],
  },
  {
    id: "image-alt-rule-group",
    packages: ["vize_l2", "vize_patina"],
    commands: [
      "clippy -p vize_patina --lib --example native_image_alt_original --example native_image_alt_canonical",
      "clippy -p vize_patina --features canonical-rule-group --lib --test canonical_rule_group --no-deps",
      "test -p vize_patina --features canonical-rule-group --lib exhausted_walk_overrides_each_rule",
      "test -p vize_patina --features canonical-rule-group --test integration native_canonical_",
      "test -p vize_patina --features canonical-rule-group --test canonical_rule_group",
      "test -p vize_l2 --features sparse-region-walk --lib",
    ],
  },
  {
    id: "jsx-module-projection",
    packages: ["vize_canon", "vize_l4"],
    commands: [
      "test -p vize_canon --features component-projection-tests --test component_prop_projection",
      "test -p vize_l4 --features module-ir --test jsx_module_projection",
    ],
  },
  {
    id: "legacy-language-server",
    packages: ["vize", "vize_canon", "vize_maestro"],
    env: { RAYON_NUM_THREADS: "4" },
    commands: [
      "clippy -p vize_canon --features legacy --lib",
      "clippy -p vize_maestro --features legacy --all-targets",
      "test -p vize_canon --features legacy --lib corsa_bridge::vue_document_package_tests::symlink_workspace::",
      "test -p vize_canon --features legacy --lib batch::virtual_project::materialize::alias_transition_tests::",
      "test -p vize_canon --features legacy --lib corsa_bridge::vue_dependencies_alias::context::cache::tests::",
      "test -p vize_canon --features legacy --lib corsa_bridge::vue_document::build::tests::",
      "test -p vize_canon --features legacy --lib lsp_client::editor_lsp::completion_resolve::issued::tests::",
      "test -p vize_canon --features legacy --lib corsa_bridge::bridge::initial_diagnostics::tests::",
      "test -p vize_maestro --features legacy --lib server::initial_diagnostics::",
      "test -p vize_maestro --features legacy --lib server::native_requests::",
      "test -p vize_maestro --features legacy --lib server::state::project_contexts::",
      "test -p vize_maestro --features legacy --lib server::importers::canonical::tests::",
      "test -p vize --features legacy --test integration -- lsp_completion_sibling_overlay_cli::",
      "test -p vize --features legacy --test integration -- lsp_symlink_workspace_staging_cli::original_include_and_open_order_controls_keep_all_authored_bytes",
      "test -p vize --features profiling --test integration -- lsp_component_attribute_authority_cli::library_refusal::original_emit_string_keeps_atomic_owned_updates_and_unsafe_name_refusals_without_file_writes --exact",
    ],
  },
  {
    id: "lexical-arrows",
    packages: ["vize_oxc_parser", "vize_l1", "vize_l2", "vize_l1_to_l2", "vize_maestro"],
    env: largeStack,
    commands: [
      "test -p vize_oxc_parser --features terminal-embedding --lib embedding::terminal::tests::",
      "clippy -p vize_l1 --features retained-expressions --lib --no-deps",
      "test -p vize_l1 --features retained-expressions --lib handoff::sequence::",
      "clippy -p vize_l2 --features lexical-arrow-initializers --lib --no-deps",
      `clippy -p vize_l1_to_l2 --features ${arrowFeatures} --lib --no-deps`,
      "clippy -p vize_maestro --features experimental-lexical-arrow-queries,micro-profiling --lib --no-deps",
      `test -p vize_l1_to_l2 --features ${arrowFeatures} --test expression_queries lexical_arrows:: -- --test-threads=1`,
      `test -p vize_l1_to_l2 --features ${arrowFeatures} --test expression_query_alloc`,
      "test -p vize_maestro --features experimental-lexical-arrow-queries,micro-profiling --lib tests::lexical:: -- --test-threads=1",
    ],
  },
  {
    id: "literal-products",
    packages: ["vize_l2", "vize_canon", "vize_patina", "vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_l2 --features literal-assignments --lib --no-deps",
      "clippy -p vize_canon --features verified-literal-assignments --lib --no-deps",
      "clippy -p vize_patina --features verified-prefer-const --lib --no-deps",
      "clippy -p vize_maestro --features experimental-literal-products --lib --no-deps",
      "test -p vize_l2 --features literal-assignments --lib query::script::assignments::",
      "test -p vize_maestro --features experimental-literal-products,micro-profiling --lib source_project::literal_products:: -- --test-threads=1",
      "clippy -p vize_l2 --features recovered-literal-prefix --lib --no-deps",
      "clippy -p vize_maestro --features experimental-recovered-literal-prefix --lib --no-deps",
      "test -p vize_l2 --features recovered-literal-prefix --lib query::script::assignments::prefix::",
      "test -p vize_maestro --features experimental-recovered-literal-prefix,micro-profiling --lib source_project::recovered_script:: -- --test-threads=1",
    ],
  },
  {
    id: "navigation-products",
    packages: ["vize_l1_to_l2", "vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_l1_to_l2 --features native-navigation-products --lib --no-deps",
      "clippy -p vize_maestro --features experimental-navigation-products,micro-profiling --lib --no-deps",
      "test -p vize_l1_to_l2 --features native-navigation-products --lib native_file::query::",
      "test -p vize_maestro --features experimental-navigation-products,micro-profiling --lib source_project::navigation::",
    ],
  },
  {
    id: "open-document-blocks",
    packages: ["vize_incremental", "vize_maestro"],
    commands: [
      "clippy -p vize_incremental --features open-block-queries --lib --example open_block_profile --no-deps",
      "clippy -p vize_maestro --features experimental-resident-blocks --lib --no-deps",
      "test -p vize_incremental --features open-block-queries --lib block_query::",
      "test -p vize_incremental --features open-block-queries --test descriptor",
      "test -p vize_maestro --features experimental-resident-blocks --lib source_project::blocks::tests",
    ],
  },
  {
    id: "plugin-template-products",
    packages: ["vize", "vize_patina", "vize_doctor", "vize_curator"],
    env: largeStack,
    commands: [
      "clippy -p vize_patina --features plugin-template-facts --lib --no-deps",
      "clippy -p vize --features plugin-project-facts --lib --no-deps",
      "clippy -p vize_curator --features inspector-template-products --lib --no-deps",
      "test -p vize_patina --features plugin-template-facts --lib plugin_batch::",
      "test -p vize_doctor --features application-analysis --test application_complexity",
      "test -p vize --features plugin-project-facts --test plugin_template_products -- --test-threads=1",
      "test -p vize_curator --features inspector-template-products --lib inspector:: -- --test-threads=1",
      "test -p vize --no-default-features --features inspector-template-products --test inspector_template_products -- --test-threads=1",
    ],
  },
  {
    id: "production-reach",
    packages: ["vize_atelier_sfc"],
    commands: [
      "test -p vize_atelier_sfc --features legacy-dom-differential --test davinci_production_reach",
    ],
  },
  {
    id: "profile-allocations",
    packages: ["vize_l0", "vize_carton"],
    commands: [
      "test -p vize_l0 --features micro-profiling --test profile_allocation_thread_law",
      "test -p vize_l0 --features micro-profiling --doc ProfileGuard",
      "test -p vize_l0 --features micro-profiling --test source_operations_profile",
      "test -p vize_carton --features micro-profiling --test davinci_profile_export per_span_allocation_counts_and_schema_validation",
    ],
  },
  {
    id: "resident-template-passes",
    packages: ["vize_incremental", "vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_incremental --features resident-template-passes --lib --no-deps",
      "test -p vize_incremental --features resident-template-passes --lib template::tests:: -- --test-threads=1",
      "test -p vize_incremental --features resident-template-passes --test resident_template_cost -- --test-threads=1",
      "clippy -p vize_maestro --features experimental-resident-template-passes --lib --no-deps",
      "test -p vize_maestro --features experimental-resident-template-passes,micro-profiling --lib source_project::template:: -- --test-threads=1",
      "test -p vize_maestro --features vize_l1_to_l2/original-template-queries --test original_template_requests",
      "clippy -p vize_maestro --no-default-features --features experimental-component-passes,vize_l1_to_l2/original-template-queries --test original_template_requests --no-deps",
      "test -p vize_maestro --no-default-features --features experimental-component-passes,vize_l1_to_l2/original-template-queries --test original_template_requests",
      "test -p vize_maestro --no-default-features --features experimental-component-passes --doc run_vue_passes",
      "clippy -p vize_maestro --features experimental-scoped-component-session,experimental-literal-products,vize_l1_to_l2/original-template-queries,vize_patina/original-template-lint --test original_template_requests --test component_tool_passes --no-deps",
      "test -p vize_maestro --features experimental-scoped-component-session,experimental-literal-products,vize_l1_to_l2/original-template-queries,vize_patina/original-template-lint --test original_template_requests --test component_tool_passes -- --test-threads=1",
    ],
  },
  {
    id: "retained-script-formatting",
    packages: ["vize_glyph", "vize_l1", "vize_l2"],
    env: largeStack,
    commands: [
      "test -p vize_glyph --features retained-script --lib script::snapshot::",
      "test -p vize_glyph --features retained-script --test auto_line_endings --test sfc_empty_script_blocks --test import_sorting --test program_snapshot",
      "test -p vize_l1 --features script-queries --lib container::vue::programs::",
      "test -p vize_l2 --features script-queries --lib query::vue::",
      "test -p vize_glyph --features retained-script,vize_l2/script-queries --test selected_script_programs",
    ],
  },
  {
    id: "retained-tsrx",
    packages: ["vize_l4", "vize_patina"],
    commands: [
      "test -p vize_l4 --features retained-tsrx --test tsrx_retained",
      "test -p vize_l4 --features retained-tsrx,typescript --test tsrx_retained",
      "test -p vize_l4 --features typescript --test tsrx_dom",
      "clippy -p vize_patina --features retained-tsrx,vize_l4/retained-tsrx --lib --test prepared_tsrx --no-deps",
      "test -p vize_patina --features retained-tsrx,vize_l4/retained-tsrx --lib linter::engine::tsrx::tests",
      "test -p vize_patina --features retained-tsrx,vize_l4/retained-tsrx --test prepared_tsrx",
    ],
  },
  {
    id: "script-binding-products",
    packages: ["vize_l2", "vize_croquis", "vize_atelier_sfc", "vize_vitrine"],
    commands: [
      "clippy --lib -p vize_l2 -p vize_croquis -p vize_atelier_sfc --features vize_atelier_sfc/script-binding-products --no-deps",
      "test -p vize_l2 --features script-declaration-queries --lib query::script::",
      "test -p vize_atelier_sfc --features script-binding-products --lib compile_script::binding_products:: -- --test-threads=1",
      "test -p vize_vitrine --no-default-features --features wasm --lib -- wasm::cross_file::effect_summary::tests::",
    ],
  },
  {
    id: "script-query-tools",
    packages: ["vize_l1", "vize_glyph", "vize_patina", "vize_canon"],
    env: largeStack,
    commands: [
      "clippy -p vize_l1 --features authored-script-queries --lib --no-deps",
      "test -p vize_l1 --features authored-script-queries --lib embed::authored_program::tests",
      "clippy -p vize_glyph --features observed-template --lib --no-deps",
      "test -p vize_glyph --features observed-template --lib formatter::",
      "clippy -p vize_patina --features authored-script-rules --lib --no-deps",
      "test -p vize_patina --features authored-script-rules --doc rules::script::authored",
      "test -p vize_patina --features authored-script-rules --test authored_script_rules -- --test-threads=1",
      "clippy -p vize_canon --features authored-script-diagnostics --lib --no-deps",
      "test -p vize_canon --features authored-script-diagnostics --lib script_parse:: -- --test-threads=1",
    ],
  },
  {
    id: "scss-products",
    packages: [
      "vize_l0",
      "vize_l1",
      "vize_l1_to_l2",
      "vize_l2",
      "vize_l4",
      "vize_atelier_sfc",
      "vize_glyph",
      "vize_maestro",
    ],
    env: largeStack,
    commands: [
      "clippy -p vize_l2 --features scss-rules --lib --no-deps",
      "test -p vize_l0 --features lsp-line-break-search --lib line_index::lsp_breaks",
      "test -p vize_l0 --features lsp-line-break-search --test lsp_line_breaks",
      "test -p vize_l1 --features independent-style-blocks --lib container::vue::descriptor::style_tests::",
      "clippy -p vize_l1_to_l2 --features component-text-condense --lib --no-deps",
      "test -p vize_l1_to_l2 --features component-text-condense --lib",
      "test -p vize_l4 --no-default-features --features expanded-css --test expanded_css",
      "test -p vize_l4 --no-default-features --features scoped-expanded-css --doc append_scoped",
      "clippy -p vize_l4 --features scoped-scss,indexed-source-maps --lib --no-deps",
      "test -p vize_l4 --features indexed-source-maps --lib write::source_map::indexed",
      "test -p vize_l4 --features indexed-source-maps,serde_json/preserve_order --lib write::source_map::indexed",
      "clippy -p vize_atelier_sfc --features native-scoped-scss --lib --no-deps",
      "test -p vize_atelier_sfc --features native-scoped-scss --test scss_products -- --test-threads=1",
      "test -p vize_atelier_sfc --features native-scoped-scss --test native_sfc_scss -- --test-threads=1",
      "clippy -p vize_atelier_sfc --features verified-scss-edits --lib --no-deps",
      "clippy -p vize_glyph --features observed-style --lib --no-deps",
      "test -p vize_glyph --features observed-style --test observed_style -- --test-threads=1",
      "clippy -p vize_maestro --features experimental-scss-definitions --lib --no-deps",
      "test -p vize_maestro --features experimental-scss-definitions --lib source_project::scss:: -- --test-threads=1",
      "clippy -p vize_maestro --features experimental-verified-style-format --lib --no-deps",
      "test -p vize_maestro --features experimental-verified-style-format --lib source_project::style_format:: -- --test-threads=1",
      "test -p vize_maestro --features experimental-verified-style-format,experimental-style-lint-products,micro-profiling --lib source_project::style_format::composition:: -- --test-threads=1",
    ],
  },
  {
    id: "style-modules",
    packages: ["vize_l1", "vize_canon", "vize_patina", "vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_patina --features observed-style-rules --lib --no-deps",
      "test -p vize_patina --features observed-style-rules --lib rules::css",
      "clippy -p vize_canon --features observed-style-modules --lib --no-deps",
      "test -p vize_canon --features observed-style-modules --lib css_module",
      "test -p vize_l1 --features observed-styles --lib css::lang::",
      "clippy -p vize_maestro --features experimental-style-lint-products,micro-profiling --lib --no-deps",
      "test -p vize_maestro --features experimental-style-lint-products,micro-profiling --lib css_module",
    ],
  },
  {
    id: "styled-projections",
    packages: ["vize_l1", "vize_l1_to_l2", "vize_l4", "vize_canon", "vize_atelier_sfc"],
    env: largeStack,
    commands: [
      "clippy -p vize_l1 -p vize_l1_to_l2 -p vize_l4 -p vize_canon -p vize_atelier_sfc --features vize_canon/original-style-binding-tests --lib --no-deps",
      "test -p vize_canon --features original-style-binding-tests --lib css_var_usage::",
      "test -p vize_canon --features original-style-binding-tests --test original_style_bindings -- --test-threads=1",
      "clippy -p vize_l1 --features style-bindings --lib --no-deps",
      "test -p vize_l1 --features style-bindings --lib css::bind::tests",
      "clippy -p vize_l4 --features vue-style-projection --lib --no-deps",
      "test -p vize_l4 --features vue-style-projection --test vue_projection",
      "clippy -p vize_canon --features styled-projection-tests --test styled_projection --example styled_projection_profile --no-deps",
      "test -p vize_canon --features styled-projection-tests --lib css_var_usage",
      "test -p vize_canon --features styled-projection-tests --test styled_projection",
    ],
  },
  {
    id: "template-passes",
    packages: ["vize_l0", "vize_l1", "vize_l1_to_l2", "vize_l2", "vize_l3", "vize_patina"],
    env: largeStack,
    commands: [
      "clippy -p vize_l0 --features streamed-pass-groups --lib --no-deps",
      "test -p vize_l0 --features streamed-pass-groups --lib pass::",
      "test -p vize_l0 --features streamed-pass-groups --test grouped_passes -- --test-threads=1",
      "test -p vize_l3 --features streamed-pass-groups --test placement",
      "test -p vize_l1 --features independent-template-block --doc CheckedTemplateBlock",
      "test -p vize_l2 --features composed-template-analysis --lib lattice::composed::",
      "clippy -p vize_l1_to_l2 --features original-template-queries --lib --no-deps",
      "test -p vize_l1_to_l2 --features original-template-queries --test original_template_queries -- --test-threads=1",
      "clippy -p vize_l1_to_l2 --features observed-template-passes --lib --no-deps",
      "test -p vize_l1_to_l2 --features observed-template-passes --test observed_template_passes -- --test-threads=1",
      "test -p vize_l1_to_l2 --features observed-template-passes --test cfg_complexity_oracle composed:: -- --test-threads=1",
      "clippy -p vize_patina --features original-template-lint --lib --no-deps",
      "test -p vize_patina --features original-template-lint --lib max_template_complexity::",
      "test -p vize_patina --features original-template-lint --test original_template_lint -- --test-threads=1",
    ],
  },
  {
    id: "template-tool-projections",
    packages: ["vize_l1_to_l2", "vize_canon", "vize_maestro"],
    env: largeStack,
    commands: [
      "clippy -p vize_l1_to_l2 --features template-operand-queries --lib --no-deps",
      "clippy -p vize_canon --features template-operand-projection --lib --no-deps",
      "clippy -p vize_maestro --features experimental-template-operand-products,micro-profiling --lib --no-deps",
      "test -p vize_canon --features template-operand-projection --lib projection::expr::",
      "test -p vize_maestro --features experimental-template-operand-products,micro-profiling --lib virtual_code::generator::",
    ],
  },
  {
    id: "versioned-fixes",
    packages: ["vize_l0", "vize_l1", "vize_maestro"],
    commands: [
      "clippy -p vize_l0 --lib --features positioned-edits --no-deps",
      "clippy -p vize_l1 --lib --features positioned-edits --no-deps",
      "clippy -p vize_maestro --features experimental-versioned-fixes --test versioned_fix_publication --no-deps",
      "test -p vize_l0 --lib --features positioned-edits line_index::",
      "test -p vize_l1 --lib --features positioned-edits,diagnostic-edit-selection edit::",
      "test -p vize_maestro --features experimental-versioned-fixes --lib source_project::edit_tests",
      "test -p vize_maestro --features experimental-versioned-fixes --test versioned_fix_publication",
    ],
  },
  {
    id: "windows-project-paths",
    packages: ["vize", "vize_canon", "vize_patina"],
    windows: true,
    runtime: true,
    env: { VIZE_TEST_REQUIRE_TSGO: "1" },
    commands: [
      "test -p vize_canon --lib batch::materialize_fs::",
      "test -p vize_patina --lib linter::corsa_session::paths",
      "test -p vize --test integration -- --exact check_removed_node_modules_package_cli::removing_a_package_between_checks_drops_its_stale_mirror_link lint_solution_tsconfig_cli::included_declarations_and_unimported_module_augmentations_join_the_lint_program lint_solution_tsconfig_cli::alias_augmentations_and_referencing_ambient_declarations_join_the_lint_program",
    ],
  },
];

/** Files whose change can alter what any row runs. */
export const rustFeatureLaneInputs: readonly string[] = [
  "tools/support/ci/rust-feature-tests.ts",
  "tools/commands/ci/rust-feature-tests.ts",
  "tools/config/pkl/parts/rust-feature-tests/job.pkl",
  ".github/workflows/pr-rust-products.yml",
  ".github/workflows/01-rust-feature-tests.yml",
];

export interface RustFeatureMatrix {
  readonly include: readonly { readonly row: string; readonly runner: string }[];
}

const linuxRunner = "blacksmith-32vcpu-ubuntu-2404";
const windowsRunner = "blacksmith-32vcpu-windows-2025";

function matrixOf(rows: readonly RustFeatureRow[]): RustFeatureMatrix {
  return {
    include: rows.map((row) => ({
      row: row.id,
      runner: row.windows ? windowsRunner : linuxRunner,
    })),
  };
}

export function allRustFeatureRows(): RustFeatureMatrix {
  return matrixOf(rustFeatureRows);
}

/**
 * Rows for one change. A lane edit or an unknown diff runs every row;
 * otherwise a row runs when the affected-crate closure reaches one of its crates.
 */
export function selectRustFeatureRows(
  paths: readonly string[],
  affectedPackages: readonly string[] | undefined,
): RustFeatureMatrix {
  if (paths.length === 0 || paths.some((file) => rustFeatureLaneInputs.includes(file)))
    return allRustFeatureRows();
  const affected = new Set(affectedPackages ?? []);
  return matrixOf(rustFeatureRows.filter((row) => row.packages.some((name) => affected.has(name))));
}

export function rustFeatureRow(id: string): RustFeatureRow {
  const row = rustFeatureRows.find((candidate) => candidate.id === id);
  if (row == null) throw new Error(`Unknown Rust feature row: ${id}`);
  return row;
}

/** Full Cargo argument lists for one row, in run order. */
export function rustFeatureCommands(row: RustFeatureRow): string[][] {
  return row.commands.map((command) => {
    const [subcommand, ...rest] = command.split(" ");
    const testArgs = rest.indexOf("--");
    const cargoArgs = testArgs < 0 ? rest : rest.slice(0, testArgs);
    const tail = testArgs < 0 ? [] : rest.slice(testArgs);
    const profile = cargoArgs.includes("--profile") ? [] : ["--profile", "ci"];
    const lints = subcommand === "clippy" ? ["--", "-D", "warnings"] : tail;
    return [subcommand!, "--locked", ...profile, ...cargoArgs, ...lints];
  });
}
