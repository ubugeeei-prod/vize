import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { LANGUAGE_PROJECT_CASE_ID } from "../../tools/support/compat/github/language-project-fixtures.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath), "utf8"));
const pairs: Array<{ source: string; before: string | null; head: string }> = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/tooling/language-project-source-pairs-10740.json"),
    "utf8",
  ),
);
const ordinaryInput = Buffer.from("unit-only ordinary input; never a product registration\n");
const ordinaryPath = "tests/_fixtures/differential/typechecker/ordinary-unit.txt";
const ordinary = {
  id: "typechecker/unit/ordinary-control",
  state: "active",
  targets: ["batch"],
  inputs: [
    {
      path: "ordinary-unit.txt",
      role: "entry",
      sha256: createHash("sha256").update(ordinaryInput).digest("hex"),
    },
  ],
};
function observe({
  rows = manifest.languageProjectCases,
  oldRows,
  missing = false,
  filler = false,
  mixed = false,
  future = false,
  unknownProtocol = false,
}: {
  rows?: unknown;
  oldRows?: unknown;
  missing?: boolean;
  filler?: boolean;
  mixed?: boolean;
  future?: boolean;
  unknownProtocol?: boolean;
} = {}) {
  const before = structuredClone(manifest),
    head = structuredClone(manifest);
  before.languageProjectCases = oldRows;
  head.languageProjectCases = rows;
  if (missing) delete head.languageProjectCases;
  if (filler) head.cases.push(ordinary);
  if (unknownProtocol) head.protocolCases.push({ id: "typechecker/protocol/unregistered-unit" });
  const paths = pairs.map((pair) => pair.source);
  if (mixed) paths.push("crates/vize_canon/src/unrelated_unit.rs");
  const result = analyzeLegacyFix({
    title: "fix(typecheck): preserve inherited project options",
    paths,
    readFile(side, file) {
      if (file === manifestPath)
        return Buffer.from(JSON.stringify(side === "base" ? before : head));
      if (file === ordinaryPath) return side === "head" ? ordinaryInput : null;
      const pair = pairs.find((pair) => pair.source === file);
      if (pair) {
        const text = side === "base" ? pair.before : pair.head;
        if (text === null) return null;
        return Buffer.from(text + (future && side === "head" && pair === pairs[0] ? "\n" : ""));
      }
      if (side === "base") return null;
      const source = path.join(root, file);
      return fs.existsSync(source) ? fs.readFileSync(source) : null;
    },
  });
  assert.equal(result.state, "fixture-required");
  assert.equal(result.products.length, 1);
  return result.products[0]!;
}

void test("the report admits only the complete language-project registration outside ordinary cases", () => {
  const result = observe();
  assert.equal(result.state, "new-inputs-registered", result.reason);
  assert.deepEqual(result.cases, [LANGUAGE_PROJECT_CASE_ID]);
});
void test("ordinary filler cannot replace a missing or stale required language-project row", () => {
  for (const options of [{ missing: true }, { rows: [] }]) {
    const result = observe({ ...options, filler: true });
    assert.equal(result.state, "invalid-manifest-or-input");
    assert.match(result.reason!, /requires one new protocol case/);
  }
  const stale = observe({ oldRows: manifest.languageProjectCases, filler: true });
  assert.equal(stale.state, "invalid-manifest-or-input");
  assert.match(stale.reason!, /requires one new protocol case/);
});
void test("mixed and future Canon changes retain their ordinary input requirement", () => {
  assert.equal(observe({ mixed: true }).state, "new-input-missing");
  const mixed = observe({ mixed: true, filler: true });
  assert.equal(mixed.state, "new-inputs-registered", mixed.reason);
  assert.deepEqual(mixed.cases, [ordinary.id, LANGUAGE_PROJECT_CASE_ID]);
  assert.equal(observe({ future: true, rows: [] }).state, "new-input-missing");
  const future = observe({ future: true, rows: [], filler: true });
  assert.equal(future.state, "new-inputs-registered", future.reason);
  assert.deepEqual(future.cases, [ordinary.id]);
});
void test("the language-project route cannot hide unknown protocol IDs", () => {
  const result = observe({ unknownProtocol: true });
  assert.equal(result.state, "invalid-manifest-or-input");
});
