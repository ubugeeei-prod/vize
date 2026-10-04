import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import {
  FOCUS_CASES_PATH,
  FOCUS_CONTRACT,
  loadFocusCases,
  validateFocusProbe,
} from "../differential/focus-history.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = loadFocusCases(root);

void test("eight exact original sources/options/tests retain fix and parent object custody", () => {
  assert.deepEqual(
    fixtures.map((fixture: any) => [
      fixture.id,
      fixture.authored.source,
      fixture.authored.test,
      fixture.authored.original_warning_count,
    ]),
    [
      [
        "autofocus-bind",
        '<input type="text" :autofocus="true" />',
        "test_invalid_has_bound_autofocus",
        1,
      ],
      [
        "accesskey-bind",
        "<div :accesskey=\"'h'\">Content</div>",
        "test_invalid_has_bound_accesskey",
        1,
      ],
      [
        "accesskey-dynamic",
        '<div :[accesskey]="shortcut">Content</div>',
        "test_valid_dynamic_accesskey_argument",
        0,
      ],
      [
        "autofocus-dynamic",
        '<input type="text" :[autofocus]="enabled" />',
        "test_valid_dynamic_autofocus_argument",
        0,
      ],
      ["autofocus-absent", '<input type="text" />', "test_valid_no_autofocus", 0],
      ["autofocus-static", '<input type="text" autofocus />', "test_invalid_has_autofocus", 1],
      ["accesskey-absent", "<div>Content</div>", "test_valid_no_accesskey", 0],
      ["accesskey-static", '<div accesskey="h">Content</div>', "test_invalid_has_accesskey", 1],
    ],
  );
  for (const fixture of fixtures) {
    const witness = fixture.authored;
    const source = fs.readFileSync(path.join(root, witness.rule_path), "utf8");
    const start = source.indexOf(`fn ${witness.test}()`);
    assert(start >= 0, witness.test);
    const next = source.indexOf("#[test]", start);
    const body = source.slice(start, next < 0 ? undefined : next);
    assert(body.includes(`r#"${witness.source}"#`) && body.includes('"test.vue"'));
    assert(
      body.includes("lint_template") &&
        body.includes(`result.warning_count, ${witness.original_warning_count}`),
    );
    const concrete = witness.rule === "a11y/no-autofocus" ? "NoAutofocus" : "NoAccessKey";
    assert(source.includes(`registry.register(Box::new(${concrete}));`));
    assert(source.includes("Linter::with_registry(registry)"));
    assert.equal(
      witness.witness_revision,
      witness.witness_kind === "parent-control" ? witness.parent : witness.fix,
    );
  }
  const constructors = fs
    .readFileSync(path.join(root, "crates/vize_patina/src/linter/config/constructors.rs"), "utf8")
    .split("pub fn with_registry(registry: RuleRegistry)")[1];
  assert(
    constructors.includes("locale: Locale::default()") &&
      constructors.includes("help_level: HelpLevel::default()"),
  );
  assert(
    constructors.includes("requested_vue_version: None") &&
      constructors.includes("requested_vapor_mode: None"),
  );
});

void test("authored input changes cannot masquerade as the reviewed original witness pack", () => {
  const original = JSON.parse(fs.readFileSync(path.join(root, FOCUS_CASES_PATH), "utf8"));
  for (const mutate of [
    (rows: any[]) => rows.pop(),
    (rows: any[]) => rows.push(rows[0]),
    (rows: any[]) => {
      rows[0].source = `<template>${rows[0].source}</template>`;
    },
    (rows: any[]) => {
      rows[0].filename = "Wrapped.vue";
    },
    (rows: any[]) => {
      rows[0].locale = "En";
    },
    (rows: any[]) => {
      rows[0].help_level = "Full";
    },
    (rows: any[]) => {
      rows[0].severity = "Warning";
    },
    (rows: any[]) => {
      rows[0].witness_blob = "0".repeat(40);
    },
    (rows: any[]) => {
      rows[0].expected = "invented complete output";
    },
  ]) {
    const rows = structuredClone(original);
    mutate(rows);
    assert.throws(() => loadFocusCases(root, Buffer.from(`${JSON.stringify(rows, null, 2)}\n`)));
  }
});

function probe(value = FOCUS_CONTRACT) {
  const bytes = Buffer.from(`${JSON.stringify(value)}\n`);
  return {
    argv: ["--contract"],
    exitStatus: 0,
    stdoutBase64: bytes.toString("base64"),
    sha256: sha256(bytes),
  };
}
const receipt = { source: { sourceRevision: "a".repeat(40) }, probes: [probe()] };
void test("probe semantics and bytes cannot claim authored defaults or completed native coverage", () => {
  validateFocusProbe(receipt);
  for (const mutation of [
    (value: any) => {
      value.options = "En/Full/Warning";
    },
    (value: any) => {
      value.acceptance = "accepted";
    },
    (value: any) => {
      value.fallback = true;
    },
    (value: any) => {
      value.registry = "default-preset";
    },
  ]) {
    const value = structuredClone(FOCUS_CONTRACT);
    mutation(value);
    assert.throws(() => validateFocusProbe({ probes: [probe(value)] }));
  }
});

void test("the first real source-built capture is retained only in full T1; pure contracts stay T0", () => {
  const execution = "tests/tooling/focus-history-capture-execution.test.ts";
  const contracts = "tests/tooling/focus-history-capture-contract.test.ts";
  const protocol = "tests/tooling/focus-history-capture-protocol.test.ts";
  for (const input of [
    FOCUS_CASES_PATH,
    "crates/vize_patina/examples/focus_history_observer/main.rs",
    "crates/vize_patina/examples/focus_history_observer/case.rs",
    "crates/vize_patina/examples/focus_history_observer/observe.rs",
    "tests/differential/focus-history.ts",
    "tests/differential/focus-history-capture.ts",
    "tests/differential/focus-history-report.ts",
    "crates/vize_patina/src/linter/config/constructors.rs",
    "crates/vize_patina/src/rules/a11y/no_autofocus.rs",
    "crates/vize_patina/src/rules/a11y/no_access_key.rs",
    "docs/davinci/decisions/2026-10-04-linter-focus-history-capture.md",
  ]) {
    const pr = planToolingTests([input], { cwd: root }).tests;
    const full = planToolingTests([input], { tier: "merge", cwd: root }).tests;
    assert(!pr.includes(execution));
    assert(pr.includes(contracts) && pr.includes(protocol));
    assert(full.includes(execution) && full.includes(contracts) && full.includes(protocol));
  }
  assert(planToolingTests([contracts], { cwd: root }).tests.includes(contracts));
});
