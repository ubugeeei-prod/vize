import assert from "node:assert/strict";

/** Exact carrier-only inverses; the caller still authenticates the entire original SHA. */
const migrations: Record<string, readonly (readonly [string, string])[]> = {
  "crates/vize/tests/support/lsp_authored_rename.rs": [
    [
      '"vizeConfig":fixture.read_file("vite.config.mjs")',
      '"vizeConfig":fixture.read_file("vize.config.json")',
    ],
  ],
  "crates/vize/tests/support/lsp_vue_project.rs": [
    ['#[path = "vite_config.rs"]\nmod vite_config;\n\n', ""],
    ['project.path().join("vite.config.mjs")', 'project.path().join("vize.config.json")'],
    ["vite_config::source(serde_json::to_vec(&json!({", "serde_json::to_vec(&json!({"],
    [
      '"languageServer": { "lint": false, "typecheck": true, "hover": true, "crossFile": cross_file }',
      '"lsp": { "lint": false, "typecheck": true, "hover": true, "crossFile": cross_file }',
    ],
    ["            .unwrap()),", "            .unwrap(),"],
  ],
  "tests/tooling/lsp-workspace-folder-changes.test.ts": [
    ['import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";\n', ""],
    [
      '  writeVueConfigFixture(rootB, JSON.stringify({ linter: { rules: { [V_FOR_KEY_RULE]: "off" } } }));',
      '  fs.writeFileSync(\n    path.join(rootB, "vize.config.json"),\n    JSON.stringify({ linter: { rules: { [V_FOR_KEY_RULE]: "off" } } }),\n    "utf8",\n  );',
    ],
  ],
  "tests/tooling/lsp-initial-diagnostics-version.test.ts": [
    ['import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";\n', ""],
    [
      "    writeVueConfigFixture(\n      workspaceDir,",
      '    fs.writeFileSync(\n      path.join(workspaceDir, "vize.config.json"),',
    ],
    [
      '          corsaPath: "./vize-missing-corsa-for-initial-sync-diagnostics",\n        },\n      }),\n    );',
      '          corsaPath: "./vize-missing-corsa-for-initial-sync-diagnostics",\n        },\n      }),\n      "utf8",\n    );',
    ],
  ],
  "crates/vize/tests/lsp_component_attribute_authority_cli/library_refusal/safe_update.rs": [
    [
      '"vizeConfig":std::fs::read_to_string(fixture.project.path().join("vite.config.mjs"))',
      '"vizeConfig":std::fs::read_to_string(fixture.project.path().join("vize.config.json"))',
    ],
  ],
  "crates/vize/tests/lsp_component_attribute_authority_cli.rs": [
    ['#[path = "support/vite_config.rs"]\nmod vite_config;\n\n', ""],
    [
      '            project.path().join("vite.config.mjs"),\n            vite_config::source(\n                serde_json::to_vec(&json!({\n                    // Preserve this historical SFC-only control\'s effective mode;\n                    // canonical Vite projects now enable JSX checking by default.\n                    "typeChecker": { "corsaPath": runtime, "jsxTypecheck": false },\n                    "lsp": { "lint": true, "typecheck": true, "hover": true, "crossFile": true }\n                }))\n                .unwrap(),\n            ),',
      '            project.path().join("vize.config.json"),\n            serde_json::to_vec(&json!({\n                "typeChecker": { "corsaPath": runtime },\n                "lsp": { "lint": true, "typecheck": true, "hover": true, "crossFile": true }\n            }))\n            .unwrap(),',
    ],
  ],
};

export function originalVueConfigFixtureSource(file: string, bytes: Uint8Array): string {
  let source = new TextDecoder("utf8", { fatal: true }).decode(bytes);
  const edits = migrations[file];
  if (!edits || !source.includes(edits[0][0])) return source;
  for (const [current, original] of edits) {
    assert.equal(source.split(current).length, 2, `one reviewed config carrier edit: ${file}`);
    source = source.replace(current, original);
  }
  return source;
}
