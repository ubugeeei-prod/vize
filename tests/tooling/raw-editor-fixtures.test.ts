import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { parse } from "yaml";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { newRawEditorCases } from "../../tools/support/compat/github/raw-editor-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = new URL("../../", import.meta.url);
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(readFileSync(new URL(manifestPath, root), "utf8"));
const row = manifest.rawEditorCases[0];
const paths: string[] = row.sources.map((source: { path: string }) => source.path);
const before = JSON.parse(
  readFileSync(new URL("tests/tooling/fixtures/raw-editor-before-images.json", root), "utf8"),
) as Record<string, string | null>;
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
type Options = {
  current?: unknown;
  previous?: unknown;
  changed?: string[];
  files?: Record<string, Buffer | null>;
  cases?: object[];
  omit?: boolean;
};
function reader(options: Options = {}) {
  return (side: "base" | "head", path: string): Buffer | null => {
    if (path === manifestPath) {
      const value = {
        ...manifest,
        cases: [...manifest.cases, ...(side === "head" ? (options.cases ?? []) : [])],
      };
      value.rawEditorCases =
        side === "base" ? (options.previous ?? []) : (options.current ?? [row]);
      if (side === "head" && options.omit) delete value.rawEditorCases;
      return Buffer.from(JSON.stringify(value));
    }
    if (Object.hasOwn(options.files ?? {}, `${side}:${path}`))
      return options.files![`${side}:${path}`]!;
    if (side === "base" && Object.hasOwn(before, path))
      return before[path] === null ? null : Buffer.from(before[path]!);
    try {
      return readFileSync(new URL(path, root));
    } catch {
      return null;
    }
  };
}
function analyze(options: Options = {}) {
  return analyzeLegacyFix({
    title: "fix(runtime): select the project compiler",
    paths: options.changed ?? paths,
    readFile: reader(options),
  }).products[0];
}
function validate(options: Options = {}) {
  return newRawEditorCases(
    options.previous ?? [],
    options.current,
    reader(options),
    options.changed ?? paths,
  );
}

void test("closed raw registration binds the complete actual source pair and all original payloads", () => {
  const result = validate({ current: [row] });
  assert.deepEqual(result.ids, [row.id]);
  assert.deepEqual([...result.sources], paths);
  assert.equal(
    loadTypecheckerManifest(new URL(manifestPath, root).pathname).cases.length,
    manifest.cases.length,
  );
});

void test("closed raw registration refuses missing, stale, malformed and forged rows", () => {
  for (const current of [
    undefined,
    [],
    null,
    [{}],
    [{ ...row, id: "typechecker/raw-editor/forged" }],
  ])
    assert.throws(() => validate({ current }));
  assert.throws(() => validate({ current: [row], previous: [row] }));
  for (const change of [
    (value: typeof row) => {
      value.state = "inactive";
    },
    (value: typeof row) => {
      value.sources.pop();
    },
    (value: typeof row) => {
      value.tests.pop();
    },
    (value: typeof row) => {
      value.tests[0].selector += "_forged";
    },
    (value: typeof row) => {
      value.input.sha256 = "0".repeat(64);
    },
  ]) {
    const current = structuredClone(row);
    change(current);
    assert.throws(() => validate({ current: [current] }));
  }
});

void test("closed raw registration refuses unreadable payloads, modified source and lost cfg(test) ownership", () => {
  const fixture =
    "tests/fixtures/typechecker/outside-import-editor-raw-10695/before-policy-diagnostics.json.txt";
  for (const value of [null, Buffer.from("[]"), Buffer.from("forged")])
    assert.throws(() => validate({ current: [row], files: { [`head:${fixture}`]: value } }));
  for (const source of paths) {
    const bytes = readFileSync(new URL(source, root));
    const changed = Buffer.concat([bytes, Buffer.from("\n// unrelated edit\n")]);
    const current = structuredClone(row);
    current.sources.find((entry: { path: string }) => entry.path === source).after = hash(changed);
    assert.throws(() => validate({ current: [current], files: { [`head:${source}`]: changed } }));
  }
  const parent = "crates/vize_canon/src/corsa_bridge.rs";
  const bytes = readFileSync(new URL(parent, root));
  assert.throws(() =>
    validate({
      current: [row],
      files: {
        [`head:${parent}`]: Buffer.from(
          bytes
            .toString()
            .replace(
              "#[cfg(test)]\nmod outside_import_editor_tests;",
              "mod outside_import_editor_tests;",
            ),
        ),
      },
    }),
  );
  assert.throws(() => validate({ current: [row], changed: paths.slice(0, 1) }));
});

// These natural report tests are intentionally red until the centrally reviewed
// report hook consumes the closed registration; helper-only success is not credit.
void test("natural report accepts only the pinned raw contract outside language counts", () => {
  assert.equal(analyze()?.state, "new-inputs-registered");
  assert.deepEqual(analyze()?.cases, [row.id]);
  assert.equal(
    analyze({ changed: [...paths, "crates/vize_canon/src/corsa_bridge/bridge.rs"] })?.state,
    "new-input-missing",
  );
});

void test("natural report cannot bypass mandatory raw custody using an ordinary filler case", () => {
  const filler = { ...manifest.cases[0], id: "typechecker/ordinary-filler-control" };
  for (const current of [[], [{ ...row, state: "inactive" }]])
    assert.notEqual(analyze({ current, cases: [filler] })?.state, "new-inputs-registered");
  assert.notEqual(analyze({ previous: [row] })?.state, "new-inputs-registered");
  assert.notEqual(analyze({ omit: true, cases: [filler] })?.state, "new-inputs-registered");
});

void test("closed raw producers remain selected by the actual Canon planner and mandatory action", () => {
  assert.ok(planNativeRuntime(paths).matrix.lane.includes("canon"));
  const action = parse(
    readFileSync(new URL(".github/actions/test-native-canon/action.yml", root), "utf8"),
  ) as {
    runs: {
      steps: Array<{
        name?: string;
        run?: string;
        if?: unknown;
        "continue-on-error"?: unknown;
        env?: Record<string, string>;
      }>;
    };
  };
  const step = action.runs.steps.find(
    (value) =>
      value.name === "Preserve the complete raw native alias contract and independent AST laws",
  );
  assert.ok(step);
  assert.equal(step.if, undefined);
  assert.equal(step["continue-on-error"], undefined);
  assert.equal(step.env?.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(
    step.env?.VIZE_TYPECHECK_REGRESSION_CAPTURE,
    "${{ runner.temp }}/native-snapshot-source/raw-alias-10695",
  );
  assert.ok(step.run?.includes("set -euo pipefail\nunset VIZE_TEST_DISABLE_TSGO\n"));
  assert.ok(
    step.run?.includes(
      "cargo test --locked --profile ci -p vize_canon --lib corsa_bridge::outside_import_editor_tests::alias_packet:: -- --nocapture --color never",
    ),
  );
  assert.ok(
    step.run?.includes(
      "cargo test --locked --profile ci -p vize_canon --lib corsa_bridge::vue_document_package_config_tests::native_editor_resolves_package_alias_from_monorepo_root -- --exact --nocapture --color never",
    ),
  );
});
