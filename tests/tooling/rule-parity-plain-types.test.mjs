import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import { buildMatrix } from "../../tools/support/compat/davinci/lib/rule-parity-matrix.ts";
import {
  derivePlainTypeDispatch,
  PLAIN_TYPE_SOURCES,
} from "../../tools/support/compat/davinci/lib/rule-parity-plain-types.ts";
import { renderArtifact } from "../../tools/support/compat/davinci/lib/rule-parity-render.ts";
import {
  matchBraceBlock,
  stripRustComments,
} from "../../tools/support/compat/davinci/lib/rule-parity-rust-text.ts";
import { stripRust } from "../../tools/support/compat/davinci/lib/rust-source.ts";

const sources = Object.fromEntries(
  Object.entries(PLAIN_TYPE_SOURCES).map(([key, relative]) => [
    key,
    fs.readFileSync(new URL(`../../crates/vize_patina/src/${relative}`, import.meta.url), "utf8"),
  ]),
);
const constants = new Map(
  [
    ...stripRustComments(sources.native).matchAll(
      /const\s+(RULE_[A-Z_]+)\s*:\s*&str\s*=\s*"([^"]+)"/gu,
    ),
  ].map((match) => [match[1], match[2]]),
);
const derive = (input = sources) => derivePlainTypeDispatch(input, constants);

function changed(key, before, after) {
  assert.ok(sources[key].includes(before), `control must change real ${key} source`);
  assert.notEqual(before, after);
  return { ...sources, [key]: sources[key].replace(before, after) };
}

function changeBody(key, name, change) {
  const source = sources[key];
  const code = stripRust(source);
  const match = new RegExp(`\\bfn\\s+${name}\\b`, "u").exec(code);
  assert.ok(match);
  const open = code.indexOf("{", match.index);
  const close = matchBraceBlock(code, open);
  const before = source.slice(open + 1, close);
  const after = change(before);
  assert.notEqual(before, after, `control must change ${name}`);
  return { ...sources, [key]: source.slice(0, open + 1) + after + source.slice(close) };
}

void test("actual plain-type source admits exactly its four extensions and two native rules", () => {
  assert.deepEqual(derive(), {
    extensions: ["ts", "mts", "cts", "tsx"],
    declarationSuffixes: [".d.ts", ".d.mts", ".d.cts"],
    rules: ["type/no-floating-promises", "type/no-reactivity-loss"],
  });
});

void test("admission metadata follows actual extension and declaration literals", () => {
  const extended = changeBody("module", "is_plain_type_script", (body) =>
    body.replace('"cts" | "tsx"', '"cts" | "tsx" | "jsx"'),
  );
  assert.deepEqual(derive(extended).extensions, ["ts", "mts", "cts", "tsx", "jsx"]);
  const declarations = changeBody("module", "is_plain_type_script", (body) =>
    body.replace(' || name.ends_with(".d.cts")', ""),
  );
  assert.deepEqual(derive(declarations).declarationSuffixes, [".d.ts", ".d.mts"]);
});

void test("JSX/TSX metadata adds only the proven native subset and preserves ordinary hook lanes", () => {
  const matrix = buildMatrix();
  const plain = [...matrix.rules.values()].filter((rule) => rule.jsxLane.includes("plain-types"));
  assert.deepEqual(plain.map((rule) => rule.name).sort(), matrix.model.plainTypes.rules);
  assert.equal(matrix.rules.size, matrix.ruleFiles.length);
  for (const name of ["type/no-unsafe-template-binding", "type/strict-boolean-expressions"]) {
    assert.equal(matrix.rules.get(name).jsx, "no");
  }
  for (const name of ["type/require-typed-props", "type/require-typed-emits"]) {
    assert.equal(matrix.rules.get(name).jsxLane, "fallback");
  }
  const artifact = renderArtifact(matrix);
  assert.match(artifact, /conditional native plain-module type rules in JSX\/TSX: 2/u);
  assert.match(artifact, /filename, not the `JsxLang` parameter/u);
  assert.match(
    artifact,
    /default opt-out, skipped inputs and unavailable checker work are not successful executions/u,
  );
  assert.doesNotMatch(artifact, /corsa session.*never invoked from `lint_jsx`/u);
  for (const rule of plain) {
    const row = artifact.split("\n").find((line) => line.startsWith(`| \`${rule.name}\` `));
    assert.match(row, /yes \(plain-types; conditional native types\)/u);
  }
});

const controls = [
  [
    "type-aware preparation guard",
    "module",
    "!linter.type_aware_enabled || !is_plain_type_script(filename)",
    "!is_plain_type_script(filename)",
  ],
  [
    "native registration",
    "module",
    "linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name)",
    "linter.is_rule_enabled(rule_name)",
  ],
  [
    "configured rule selection",
    "module",
    "linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name)",
    "linter.registry.has_rule(rule_name)",
  ],
  [
    "promise query branch",
    "module",
    "push_promise_slots(source, filename, &mut slots);",
    "missing_promise_slots(source, filename, &mut slots);",
  ],
  [
    "reactivity query branch",
    "module",
    "push_reactivity_slots(linter, source, filename, &mut result, &mut slots);",
    "missing_reactivity_slots(linter, source, filename, &mut result, &mut slots);",
  ],
  ["query-free input", "module", "if slots.is_empty()", "if false"],
  [
    "query-free single-file completion",
    "module",
    "if prepared.virtual_content().is_some()",
    "if true",
  ],
  ["single-file checker evaluation", "module", "prepared.evaluate(linter, session)", "Ok(())"],
  [
    "prepared checker evaluation",
    "prepared",
    "session.probe_types_at_offsets(&plan.content, &requests)?",
    "Vec::new()",
  ],
  [
    "prepared diagnostic delivery",
    "prepared",
    "report_probe(&mut self.result, kind, probe.as_ref());",
    "ignore_probe(kind);",
  ],
  [
    "promise diagnostic identity",
    "module",
    "LintDiagnostic::warn(\n                        RULE_NO_FLOATING_PROMISES,",
    "LintDiagnostic::warn(\n                        RULE_NO_REACTIVITY_LOSS,",
  ],
  [
    "reactivity diagnostic identity",
    "reactivity",
    "LintDiagnostic::warn(\n            RULE_NO_REACTIVITY_LOSS,",
    "LintDiagnostic::warn(\n            RULE_NO_FLOATING_PROMISES,",
  ],
  [
    "native-only module",
    "linter",
    '#[cfg(not(target_arch = "wasm32"))]\nmod native_type_aware;',
    "mod native_type_aware;",
  ],
  ["native-only batch", "engine", '#[cfg(not(target_arch = "wasm32"))]\nmod batch;', "mod batch;"],
  [
    "host opt-in",
    "config",
    "self.type_aware_enabled = enabled;",
    "self.type_aware_enabled = true;",
  ],
  ["explicit type-rule opt-in", "config", "if is_type_rule(rule_name)", "if false"],
  [
    "enabled type-rule opt-in",
    "selection",
    "if rules.as_ref().is_some_and(|rules| has_type_rule(rules))",
    "if false",
  ],
  ["additional type-rule opt-in", "selection", "if has_type_rule(&rules)", "if false"],
  ["default opt-out", "constructors", "type_aware_enabled: false,", "type_aware_enabled: true,"],
  ["configured disable", "config", "if self.disabled_rules.contains(rule_name)", "if false"],
  ["enabled-rule filter", "config", "Some(set) => set.contains(rule_name)", "Some(_set) => true"],
  [
    "project validation",
    "native",
    "super::corsa_session::validate_tsconfig(&project_root, tsconfig).err()",
    "None",
  ],
  ["runtime failure", "native", ".map_err(TypeAwareFailure::Runtime)", ".map_err(ignore_runtime)"],
];
for (const [name, key, before, after] of controls) {
  void test(`source contract rejects missing ${name}`, () => {
    assert.throws(() => derive(changed(key, before, after)), /plain type dispatch drift/u);
  });
}

for (const method of ["prepare_script", "prepare_jsx"]) {
  void test(`owned ${method} must reach the same plain-module preparation`, () => {
    const modified = changeBody("batch", method, (body) =>
      body.replace("plain_module::prepare_plain_module", "plain_module::missing_preparation"),
    );
    assert.throws(() => derive(modified), /bypasses plain-module preparation/u);
  });
}

void test("public JSX completion retains its actual native-only delegate", () => {
  const modified = changeBody("engine", "lint_jsx", (body) =>
    body.replace('#[cfg(not(target_arch = "wasm32"))]', ""),
  );
  assert.throws(() => derive(modified), /public JSX completion/u);
});

void test("declaration exclusion, filename boundaries and query-free return cannot silently drift", () => {
  for (const change of [
    (body) => body.replace("return false", "return true"),
    (body) => body.replace("unwrap_or(filename)", "unwrap_or(other_filename)"),
    (body) => body.replace('"tsx"', '"tsx" | "tsx"'),
    (body) => body.replace("rsplit_once('.')", "rsplit_once(':')"),
  ]) {
    assert.throws(
      () => derive(changeBody("module", "is_plain_type_script", change)),
      /plain type dispatch drift/u,
    );
  }
  const noQuery = changeBody("prepared", "evaluate", (body) =>
    body.replace("return Ok(())", "return Err(missing_query)"),
  );
  assert.throws(() => derive(noQuery), /query-free evaluation/u);
});

void test("comments and literals cannot manufacture branch evidence", () => {
  const modified = changeBody("module", "prepare_plain_module", (body) =>
    body.replace(
      "push_promise_slots(source, filename, &mut slots);",
      'let text = r#"push_promise_slots(source, filename, &mut slots);"#; // push_promise_slots(source, filename, &mut slots);',
    ),
  );
  assert.throws(() => derive(modified), /query branches changed order or reachability/u);
  const harmless = {
    ...sources,
    module:
      sources.module +
      '\n // fn is_plain_type_script() { fake }\n const TEXT: &str = r#"fn is_plain_type_script() { fake }"#;\n',
  };
  assert.deepEqual(derive(harmless), derive());
});

void test("dead or reordered real query branches cannot manufacture admitted membership", () => {
  for (const change of [
    (body) => `if false { ${body} } unreachable!()`,
    (body) =>
      body.replace("let mut slots = Vec::new();", "return unused; let mut slots = Vec::new();"),
    (body) => body.replace("if promises {", "if reactivity {"),
  ]) {
    assert.throws(
      () => derive(changeBody("module", "prepare_plain_module", change)),
      /query branches changed order or reachability/u,
    );
  }
  const bypass = changeBody("engine", "lint_jsx", (body) =>
    body.replace("#[cfg", "return result; #[cfg"),
  );
  assert.throws(() => derive(bypass), /public JSX completion changed order or reachability/u);
});

void test("ambiguous helpers and missing native rule constants fail closed", () => {
  assert.throws(
    () => derive({ ...sources, module: sources.module + "\nfn is_plain_type_script() {}\n" }),
    /expected one is_plain_type_script/u,
  );
  const incomplete = new Map(constants);
  incomplete.delete("RULE_NO_REACTIVITY_LOSS");
  assert.throws(
    () => derivePlainTypeDispatch(sources, incomplete),
    /unknown rule RULE_NO_REACTIVITY_LOSS/u,
  );
});
