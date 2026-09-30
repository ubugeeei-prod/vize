import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { readFileSync } from "node:fs";

import {
  analyzeLegacyFix,
  formatSummary,
  legacyProducts,
} from "../../tools/support/compat/github/report-legacy-fix-fixtures.mjs";

const input = Buffer.from("<template><div /></template>\n");
const caseRow = {
  id: "formatter/sfc/new-fix",
  state: "active",
  targets: ["fmt"],
  inputs: {
    root: "new-fix",
    files: [{ path: "App.vue", sha256: createHash("sha256").update(input).digest("hex") }],
  },
};
const manifest = (cases) =>
  Buffer.from(
    JSON.stringify({
      schema: "vize.differential.manifest",
      version: 1,
      product: "formatter",
      cases,
    }),
  );
const source = ["crates/vize_glyph/src/formatter.rs"];

function report({ title = "fix(format): preserve input", before, after, files = {} } = {}) {
  const tree = new Map([
    ["base:tests/_fixtures/differential/formatter/manifest.json", before ?? null],
    ["head:tests/_fixtures/differential/formatter/manifest.json", after ?? null],
    ...Object.entries(files),
  ]);
  return analyzeLegacyFix({
    title,
    paths: source,
    readFile: (side, path) => tree.get(`${side}:${path}`) ?? null,
  });
}

void test("known legacy product source paths are mapped without counting tests or level crates", () => {
  assert.deepEqual(
    legacyProducts([
      "crates/vize_atelier_sfc/src/style.rs",
      "crates/vize_armature/src/parser.rs",
      "crates/vize_glyph/src/formatter.rs",
      "crates/vize_patina/src/rules.rs",
      "crates/vize_canon/src/check.rs",
      "crates/vize_maestro/src/server.rs",
      "davinci/vize_l1/src/lib.rs",
      "crates/vize_glyph/tests/fix.rs",
    ]),
    ["compiler", "formatter", "linter", "lsp", "typechecker"],
  );
});

void test("unregistered fixes are reported without granting parity", () => {
  assert.deepEqual(report().products, [
    { product: "formatter", state: "adapter-or-manifest-missing", cases: [] },
  ]);
  const unchanged = report({ before: manifest([caseRow]), after: manifest([caseRow]) });
  assert.equal(unchanged.products[0].state, "new-input-missing");
  assert.match(formatSummary(unchanged), /grants no parity or native acceptance/);
});

void test("only a newly registered active case with a matching pinned input counts", () => {
  const path = "head:tests/_fixtures/differential/formatter/new-fix/App.vue";
  const result = report({
    before: manifest([]),
    after: manifest([caseRow]),
    files: { [path]: input },
  });
  assert.deepEqual(result.products, [
    { product: "formatter", state: "new-inputs-registered", cases: [caseRow.id] },
  ]);
  assert.equal(
    report({
      before: manifest([]),
      after: manifest([caseRow]),
      files: { [path]: Buffer.from("wrong") },
    }).products[0].state,
    "invalid-manifest-or-input",
  );
  assert.equal(
    report({
      before: manifest([]),
      after: manifest([{ ...caseRow, targets: [] }]),
      files: { [path]: input },
    }).products[0].state,
    "invalid-manifest-or-input",
  );
});

void test("non-fix titles and native-only paths do not claim legacy fixture requirements", () => {
  assert.equal(report({ title: "refactor(format): move printer" }).state, "not-a-fix-pr");
  assert.equal(
    analyzeLegacyFix({
      title: "fix(l2): correct lowering",
      paths: ["davinci/vize_l1_to_l2/src/lower.rs"],
      readFile: () => {
        throw new Error("should not read a manifest");
      },
    }).state,
    "no-known-legacy-source",
  );
});

void test("the Actions planner reports on PRs without adding a required gate", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url),
    "utf8",
  );
  assert.match(
    workflow,
    /- name: Report legacy fix differential inputs\n\s+if: \$\{\{ github\.event_name == 'pull_request' \}\}\n\s+continue-on-error: true/,
  );
  assert.match(
    workflow,
    /run: node tools\/support\/compat\/github\/report-legacy-fix-fixtures\.mjs/,
  );
});
