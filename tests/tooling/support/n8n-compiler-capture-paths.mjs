export const productSource = "72aac62d7c2932c71075d579c76459b270d909da";
export const example = "crates/vize_atelier_sfc/examples/n8n_compiler_custody/main.rs";
export const exampleSupport = "crates/vize_atelier_sfc/examples/n8n_compiler_custody/native.rs";
export const cases = "tests/_fixtures/differential/compiler/n8n-adoption/cases.json";
export const official = "tools/support/compat/n8n-official-compiler-custody.mjs";
const fixture = "tests/_fixtures/differential/compiler";
const modes = [
  "template-function-no-prefix-map-false",
  "template-module-prefix-map-false",
  "template-real-bindings-function-no-prefix-map-false",
  "template-real-bindings-module-prefix-map-false",
  "template-function-no-prefix-map-true",
  "template-module-prefix-map-true",
  "template-real-bindings-function-no-prefix-map-true",
  "template-real-bindings-module-prefix-map-true",
];
export const capturePaths = [
  "crates/vize_atelier_sfc/Cargo.toml",
  example,
  exampleSupport,
  cases,
  official,
  ...[".gitattributes", "expected.json", "provenance.json", "template.vue.txt"].map(
    (name) => `${fixture}/n8n-default-slot-loop/${name}`,
  ),
  ...[
    ".gitattributes",
    "InstanceAiConfirmationPanel.vue.txt",
    "LICENSE.md.txt",
    "LICENSE_EE.md.txt",
    "diagnostics.json",
    "provenance.json",
    "retained-controls.json",
  ].map((name) => `${fixture}/n8n-if-key-regression/${name}`),
  ...[
    "record.json",
    "template.txt",
    ...modes.flatMap((name) => [
      `${name}.json`,
      `${name}.${name.includes("real-bindings") ? "ts" : "js"}`,
    ]),
  ].map((name) => `${fixture}/n8n-if-key-regression/official/${name}`),
].sort();
export const driverPaths = capturePaths.filter(
  (name) => name !== "crates/vize_atelier_sfc/Cargo.toml",
);
