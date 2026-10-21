import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  canonicalDialects,
  classifyDialectCoverage,
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
  const manifests = Object.fromEntries(products.map((product) => [product, load(product)]));
  const summaries = Object.fromEntries(
    products.map((product) => [product, summarizeDialectCoverage(manifests[product].cases)]),
  );
  const custody = Object.fromEntries(
    products.map((product) => [
      product,
      manifests[product].cases.map((fixture) => ({
        id: fixture.id,
        ...classifyDialectCoverage(fixture.dialectCoverage),
      })),
    ]),
  );
  assert.deepEqual(
    Object.fromEntries(products.map((product) => [product, summaries[product].plannedCases])),
    Object.fromEntries(products.map((product) => [product, custody[product].length])),
  );
  assert.deepEqual(
    Object.fromEntries(products.map((product) => [product, summaries[product].provisionalCases])),
    Object.fromEntries(
      products.map((product) => [
        product,
        custody[product].filter((fixture) => fixture.measurement === "provisional").length,
      ]),
    ),
  );
  const plannedCases = Object.values(summaries).reduce(
    (count, summary) => count + summary.plannedCases,
    0,
  );
  assert.equal(
    Object.values(summaries).reduce((count, summary) => count + summary.unknownReadinessCases, 0),
    plannedCases,
    "source inspection must not reduce the zero-fallback readiness denominator",
  );
  for (const product of products) {
    assert.equal(
      new Set(custody[product].map((fixture) => fixture.id)).size,
      custody[product].length,
    );
    for (const measurement of ["provisional", "verified"]) {
      const dialects = {};
      for (const fixture of custody[product].filter((row) => row.measurement === measurement)) {
        for (const dialect of fixture.dialects) dialects[dialect] = (dialects[dialect] ?? 0) + 1;
      }
      assert.deepEqual(summaries[product][measurement], dialects, `${product} ${measurement}`);
    }
  }
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

void test("typechecker coverage records every pinned authored source dialect", () => {
  const source = load("typechecker");
  const summary = summarizeDialectCoverage(source.cases);
  assert.deepEqual(summary, {
    plannedCases: 68,
    verifiedCases: 0,
    provisionalCases: 67,
    unknownReadinessCases: 68,
    verified: {},
    provisional: {
      "html-template": 66,
      ts: 67,
      tsx: 3,
      "vue3-sfc": 67,
    },
  });
  assert.deepEqual(
    source.cases
      .filter((fixture) => fixture.dialectCoverage.state === "unknown")
      .map((fixture) => fixture.id),
    ["typechecker/nuxt-plugin-injection-types/authored-dependency-identity"],
  );
});
