import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  canonicalDialects,
  loadProductManifest,
  normalizeDialectCoverage,
  summarizeDialectCoverage,
} from "../differential/harness.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const products = ["compiler", "formatter", "linter", "lsp", "typechecker"];

function load(product) {
  return loadProductManifest(
    path.join(root, `tests/_fixtures/differential/${product}/manifest.json`),
    product,
  );
}

void test("every product manifest contributes to the case-level dialect denominator", () => {
  const summaries = Object.fromEntries(
    products.map((product) => [product, summarizeDialectCoverage(load(product).cases)]),
  );
  assert.deepEqual(
    Object.fromEntries(products.map((product) => [product, summaries[product].plannedCases])),
    { compiler: 12, formatter: 18, linter: 59, lsp: 27, typechecker: 47 },
  );
  assert.deepEqual(
    Object.fromEntries(products.map((product) => [product, summaries[product].provisionalCases])),
    { compiler: 12, formatter: 16, linter: 0, lsp: 0, typechecker: 0 },
  );
  assert.equal(
    Object.values(summaries).reduce((count, summary) => count + summary.plannedCases, 0),
    163,
  );
  assert.equal(
    Object.values(summaries).reduce((count, summary) => count + summary.unknownReadinessCases, 0),
    163,
    "source inspection must not reduce the zero-fallback readiness denominator",
  );
  assert.deepEqual(summaries.compiler.provisional, {
    "html-template": 12,
    ts: 1,
    "vue3-sfc": 12,
  });
  assert.deepEqual(summaries.compiler.verified, {});
});

void test("coverage schema rejects malformed and noncanonical verified claims", () => {
  const valid = {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "source-inspection",
        dialects: ["vue3-sfc", "html-template"],
        selector: "default Vue 3 dialect",
      },
    ],
  };
  assert.deepEqual(normalizeDialectCoverage(valid, "fixture"), valid);
  for (const [change, error] of [
    [(value) => (value.state = "complete"), /invalid coverage state/],
    [(value) => (value.evidence = []), /needs evidence/],
    [(value) => value.evidence[0].dialects.push("vue3-sfc"), /duplicate dialect/],
    [(value) => (value.evidence[0].dialects = ["Vue 3"]), /invalid dialect label/],
    [(value) => (value.extra = true), /coverage schema is closed/],
    [(value) => (value.evidence[0].extra = true), /evidence schema is closed/],
    [
      (value) => {
        value.state = "verified";
        value.evidence[0].dialects = ["html-template"];
      },
      /verified dialect must be canonical/,
    ],
  ]) {
    const value = structuredClone(valid);
    change(value);
    assert.throws(() => normalizeDialectCoverage(value, "fixture"), error);
  }
  const verified = structuredClone(valid);
  verified.state = "verified";
  verified.evidence[0].dialects = [...canonicalDialects];
  normalizeDialectCoverage(verified, "fixture");
});

void test("missing case coverage becomes explicit unknown without changing manifest bytes", () => {
  const source = load("lsp");
  assert(source.cases.every((fixture) => fixture.dialectCoverage.state === "unknown"));
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dialect-ledger-"));
  try {
    const file = path.join(temporary, "manifest.json");
    fs.copyFileSync(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"), file);
    assert.equal(loadProductManifest(file, "lsp").manifestSha256, source.manifestSha256);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
