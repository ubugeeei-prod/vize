import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  analyzeLegacyFix,
  enforced,
  failingProducts,
  formatSummary,
  GATE_SINCE,
  legacyProducts,
} from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const input = Buffer.from("<template><div /></template>\n");
const caseRow = {
  id: "formatter/sfc/new-fix",
  state: "active",
  targets: ["fmt"],
  inputs: { root: "new-fix", files: [{ path: "App.vue", sha256: sha(input) }] },
};
const manifest = (product: string, cases: object[]) =>
  Buffer.from(JSON.stringify({ schema: "vize.differential.manifest", version: 1, product, cases }));
const source = ["crates/vize_glyph/src/formatter.rs"];

function report({
  title = "fix(format): preserve input",
  before,
  after,
  files = {},
}: {
  title?: string;
  before?: Buffer;
  after?: Buffer;
  files?: Record<string, Buffer>;
} = {}) {
  const tree = new Map<string, Buffer | null>([
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

void test("a fix without a new pinned input fails the requirement", () => {
  const missing = report();
  assert.deepEqual(missing.products, [
    { product: "formatter", state: "adapter-or-manifest-missing", cases: [] },
  ]);
  assert.deepEqual(failingProducts(missing), ["formatter"]);
  const unchanged = report({
    before: manifest("formatter", [caseRow]),
    after: manifest("formatter", [caseRow]),
  });
  assert.equal(unchanged.products[0]?.state, "new-input-missing");
  assert.match(formatSummary(unchanged, true), /^Required: .*Native parity is not checked here/m);
  assert.match(formatSummary(unchanged, true), /Add the fixed input/);
});

void test("only a newly registered active case with a matching pinned input passes", () => {
  const path = "head:tests/_fixtures/differential/formatter/new-fix/App.vue";
  const added = (row: object, bytes: Buffer) =>
    report({
      before: manifest("formatter", []),
      after: manifest("formatter", [row]),
      files: { [path]: bytes },
    });
  const passing = added(caseRow, input);
  assert.deepEqual(passing.products, [
    { product: "formatter", state: "new-inputs-registered", cases: [caseRow.id] },
  ]);
  assert.deepEqual(failingProducts(passing), []);
  assert.equal(
    added(caseRow, Buffer.from("wrong")).products[0]?.state,
    "invalid-manifest-or-input",
  );
  assert.equal(
    added({ ...caseRow, targets: [] }, input).products[0]?.state,
    "invalid-manifest-or-input",
  );
});

void test("a new compiler input also requires a pinned full-module reference", () => {
  const reference = Buffer.from("export default {}\n");
  const compilerCase = {
    id: "compiler/sfc/new-fix",
    state: "active",
    targets: ["ssr"],
    inputs: {
      root: "new-fix",
      files: [{ path: "Layout.vue.txt", sha256: sha(input), role: "entry" }],
    },
    adapters: {
      legacy: "compiler-cli-ssr-v1",
      native: null,
      reasons: { native: "whole-product native compiler path unavailable" },
    },
    comparison: {
      contract: "compiler-ssr-module-bytes-v1",
      requiredFacets: ["code", "css", "errors", "warnings", "macro_artifacts"],
    },
    reference: {
      path: "new-fix/module.expected.txt",
      sha256: sha(reference),
      source: "crates/vize_atelier_sfc/tests/example.rs:full_module",
    },
  };
  const root = "head:tests/_fixtures/differential/compiler";
  const files: Record<string, Buffer | null> = {
    [`${root}/manifest.json`]: manifest("compiler", [compilerCase]),
    [`${root}/new-fix/Layout.vue.txt`]: input,
    [`${root}/new-fix/module.expected.txt`]: reference,
  };
  const analyze = (overrides: Record<string, Buffer | null> = {}) =>
    analyzeLegacyFix({
      title: "fix(sfc): preserve SSR module",
      paths: ["crates/vize_atelier_sfc/src/compile.rs"],
      readFile: (side, path) => ({ ...files, ...overrides })[`${side}:${path}`] ?? null,
    }).products[0]?.state;
  assert.equal(analyze(), "new-inputs-registered");
  const drifted = { [`${root}/new-fix/module.expected.txt`]: Buffer.from("drift") };
  assert.equal(analyze(drifted), "invalid-manifest-or-input");
  assert.equal(
    analyze({ [`${root}/new-fix/module.expected.txt`]: null }),
    "invalid-manifest-or-input",
  );
  const wrongAdapter = structuredClone(compilerCase);
  wrongAdapter.adapters.legacy = "unregistered-compiler";
  assert.equal(
    analyze({ [`${root}/manifest.json`]: manifest("compiler", [wrongAdapter]) }),
    "invalid-manifest-or-input",
  );
});

void test("non-fix titles and native-only paths are not required to add inputs", () => {
  const refactor = report({ title: "refactor(format): move printer" });
  assert.equal(refactor.state, "not-a-fix-pr");
  assert.deepEqual(failingProducts(refactor), []);
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

void test("PRs opened before the requirement started stay report-only", () => {
  const before = new Date(Date.parse(GATE_SINCE) - 1000).toISOString();
  assert.equal(enforced(before), false);
  assert.equal(enforced(GATE_SINCE), true);
  assert.equal(enforced("2026-12-01T00:00:00Z"), true);
  assert.equal(enforced(undefined), true);
  assert.match(formatSummary(report(), false), /^Report only: .*opened before/m);
  assert.doesNotMatch(formatSummary(report(), false), /Add the fixed input/);
});

void test("the PR planner enforces the requirement on pull requests", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url),
    "utf8",
  );
  const step = /- name: Require legacy fix differential inputs\n((?:\s{8,}.*\n)+)/.exec(workflow);
  assert.ok(step, "legacy fix input step");
  const body = step[1] ?? "";
  assert.match(body, /if: \$\{\{ github\.event_name == 'pull_request' \}\}/);
  assert.doesNotMatch(body, /continue-on-error/);
  assert.match(body, /PR_CREATED_AT: \$\{\{ github\.event\.pull_request\.created_at \}\}/);
  assert.match(body, /run: node tools\/support\/compat\/github\/report-legacy-fix-fixtures\.ts/);
});
