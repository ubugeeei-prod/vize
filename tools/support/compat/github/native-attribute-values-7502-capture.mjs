// Affected PRs capture genuine original values, whole modules and runtime evidence.
// Merge groups invoke the action unconditionally; prose is not a source admission.
const sourceTrees = [
  "davinci/vize_l0/src",
  "davinci/vize_l0_derive/src",
  "davinci/vize_l1/src/markup",
  "davinci/vize_l1/src/parse",
  "davinci/vize_l1/src/container/vue",
  "davinci/vize_l1/src/embed",
  "davinci/vize_l1_to_l2/src/native",
  "davinci/vize_l1_to_l2/src/native_file",
  "davinci/vize_l1_to_l2/src/vue_file",
  "davinci/vize_l2/src/file",
  "davinci/vize_l2/src/artifact",
  "davinci/vize_l2/src/op",
  "davinci/vize_l2/src/lang/js/file/native",
  "davinci/vize_l3/src/decision",
  "davinci/vize_l4/src/write",
  "davinci/vize_l4/src/runtime",
  "davinci/vize_l4/src/module",
  "davinci/vize_l4/src/targets/dom",
  "davinci/vize_l4/src/targets/ssr",
  "davinci/vize_l4/src/targets/vapor",
  "crates/vize_atelier_sfc/src/native",
  "crates/vize_atelier_sfc/src/native_selected",
  "crates/vize_atelier_sfc/src/native_ssr",
  "crates/vize_atelier_sfc/src/native_vapor",
  "crates/vize_atelier_sfc/tests/native_attribute_values_7502",
  "vendor/oxc_parser/src",
  "vendor/oxc_parser_compat/src",
];
const exactInputs = new Set([
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  ".cargo/config.toml",
  "package.json",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
  "npm/ui/package.json",
  "npm/plugin-sdk/package.json",
  "npm/plugin-sdk/sandbox.js",
  "npm/plugin-sdk/sandbox-worker.js",
  "npm/plugin-sdk/identity.js",
  ...[
    "vize_l0",
    "vize_l0_derive",
    "vize_l1",
    "vize_l1_to_l2",
    "vize_l2",
    "vize_l3",
    "vize_l4",
  ].flatMap((crate) => [`davinci/${crate}/Cargo.toml`, `davinci/${crate}/src/lib.rs`]),
  "davinci/vize_l1/src/container.rs",
  "davinci/vize_l2/src/lang.rs",
  "davinci/vize_l2/src/lang/js.rs",
  "davinci/vize_l2/src/lang/js/file.rs",
  "davinci/vize_l4/src/targets.rs",
  "davinci/vize_l4/src/expr.rs",
  "crates/vize_atelier_sfc/Cargo.toml",
  "crates/vize_atelier_sfc/src/lib.rs",
  "vendor/oxc_parser/Cargo.toml",
  "vendor/oxc_parser_compat/Cargo.toml",
  "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/original_inputs.json",
  "crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/reviewed_output.json",
  "tools/support/compat/davinci/plugin-sandbox-image.mjs",
  "tools/support/compat/github/native-attribute-values-7502-capture.mjs",
  "tools/support/compat/github/plan-tooling-tests.mjs",
  "tools/config/vite-plus/tooling-test-scopes.ts",
  ".github/actions/test-native-attribute-values-7502/action.yml",
  ".github/workflows/pr-source-checks.yml",
  ".github/workflows/check.yml",
]);
export function nativeAttributeValues7502CaptureRequired(paths) {
  return paths.some(
    (path) =>
      exactInputs.has(path) ||
      sourceTrees.some(
        (tree) => path === `${tree}.rs` || (path.startsWith(`${tree}/`) && path.endsWith(".rs")),
      ) ||
      /^tests\/tooling\/native-attribute-values-7502-[a-z0-9-]+\.test\.ts$/.test(path) ||
      /^tests\/tooling\/support\/native-attribute-values-7502-[a-z0-9-]+\.ts$/.test(path),
  );
}
