import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import { copyBundle, readBundles, writeBundleEntry } from "../differential/bundle.ts";
import { loadProductManifest } from "../differential/harness.mjs";
import {
  buildCensus,
  renderCensusMarkdown,
  verifyZeroFallback,
} from "../differential/native-acceptance.ts";

const sourceRevision = "a".repeat(40);
const buildReceiptSha256 = "b".repeat(64);
const policy = {
  schema: "vize.deletion-readiness.policy",
  version: 1,
  dialects: ["vue3-sfc", "pug-template"],
  targets: { compiler: ["dom", "ssr"] },
};
const verified = {
  state: "verified",
  registryRef: null,
  evidence: [{ kind: "execution", dialects: ["vue3-sfc"] }],
};
const native = {
  state: "completed",
  observation: { ok: true },
  provenance: {
    scope: "whole-product",
    sourceRevision,
    buildReceiptSha256,
    contributions: [
      { stage: "parse", implementation: "native", factOrigin: "native", fallback: false },
      { stage: "emit", implementation: "native", factOrigin: "native", fallback: false },
    ],
  },
};
const options = {
  sourceRevision,
  buildReceiptSha256,
  requiredStages: ["parse", "emit"],
  verifyObservation: (observation: { ok?: boolean }) => observation?.ok === true,
  verifyComparison: (row: { comparison: { state: string } }) => row.comparison.state === "equal",
};

type Row = { id: string; target: string; native: object; comparison: { state: string } };
function pack(root: string, rows: Row[], revision = sourceRevision) {
  const manifestPath = path.join(root, `manifest-${fs.readdirSync(root).length}.json`);
  const ids = [...new Set(rows.map((row) => row.id))];
  fs.writeFileSync(
    manifestPath,
    JSON.stringify({
      schema: "vize.differential.manifest",
      version: 1,
      product: "compiler",
      baseRevision: revision,
      cases: ids.map((id) => ({
        id,
        state: "active",
        targets: [...new Set(rows.filter((row) => row.id === id).map((row) => row.target))],
        dialectCoverage: verified,
      })),
    }),
  );
  const loaded = loadProductManifest(manifestPath, "compiler");
  const report = {
    schema: "vize.differential.result",
    version: 1,
    product: "compiler",
    manifestSha256: loaded.manifestSha256,
    sourceRevision: revision,
    rows: rows.map((row) => ({ ...row, legacy: { state: "completed" } })),
    summary: { plannedCases: ids.length },
  };
  const acceptance = summarizeNativeAcceptance(loaded, report, {
    ...options,
    sourceRevision: revision,
  });
  return { manifestPath, loaded, report, acceptance };
}

function withRoot(run: (root: string) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-acceptance-"));
  try {
    run(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

const mixedRows: Row[] = [
  { id: "compiler/sfc/one", target: "dom", native, comparison: { state: "equal" } },
  { id: "compiler/sfc/one", target: "ssr", native, comparison: { state: "different" } },
  {
    id: "compiler/sfc/two",
    target: "dom",
    native: { state: "unsupported" },
    comparison: { state: "not-compared" },
  },
  {
    id: "compiler/sfc/two",
    target: "ssr",
    native: { state: "legacy-backed" },
    comparison: { state: "not-compared" },
  },
];

void test("bundle entries round-trip and publish every fallback and unmeasured coordinate", () => {
  withRoot((root) => {
    const bundle = path.join(root, "bundle");
    const entry = pack(root, mixedRows);
    const dir = writeBundleEntry({
      repoRoot: root,
      bundleDir: bundle,
      tier: "T1",
      pack: "cli",
      ...entry,
    });
    assert.equal(dir, path.join(bundle, "T1", "compiler", "cli"));
    const entries = readBundles([bundle]);
    assert.deepEqual(
      entries.map(({ tier, product, pack: name, sourceRevision: revision }) => [
        tier,
        product,
        name,
        revision,
      ]),
      [["T1", "compiler", "cli", sourceRevision]],
    );
    assert.deepEqual(entries[0].loaded.manifest.sourceManifest, {
      path: path.basename(entry.manifestPath),
      sha256: entry.loaded.manifestSha256,
    });
    const census = buildCensus(entries, policy, "c".repeat(64));
    assert.deepEqual(census.products.compiler.targets.dom.T1, {
      planned: 2,
      nativeHandled: 1,
      nativeEquivalent: 1,
      unsupported: 1,
      legacyBacked: 0,
      unverified: 0,
    });
    assert.deepEqual(census.products.compiler.targets.ssr.total, {
      planned: 2,
      nativeHandled: 1,
      nativeEquivalent: 0,
      unsupported: 0,
      legacyBacked: 1,
      unverified: 0,
    });
    const verdict = verifyZeroFallback(census, policy);
    assert.equal(verdict.holds, false);
    assert.deepEqual(verdict.blockers, [
      { product: "compiler", code: "legacyFallbacks", detail: "2 coordinate(s)" },
      { product: "compiler", code: "nonEquivalent", detail: "1 coordinate(s)" },
      { product: "compiler", code: "unmeasured-target", detail: "T2/dom" },
      { product: "compiler", code: "unmeasured-target", detail: "T2/ssr" },
      { product: "compiler", code: "unmeasured-dialect", detail: "pug-template" },
    ]);
    assert.equal(
      renderCensusMarkdown(census, verdict),
      [
        "### Native-only acceptance per product",
        "",
        "Sources: T1 `aaaaaaaaaaaa`, T2 `absent`. T1 is the in-repo fixture corpus, T2 the real-project corpus.",
        "",
        "Product | Target | T1 native-only | T2 native-only | Equivalent | Legacy-backed | Unsupported | Unverified",
        "--- | --- | ---: | ---: | ---: | ---: | ---: | ---:",
        "compiler | dom | 1/2 (50.0%) | unmeasured | 1/2 | 0 | 1 | 0",
        "compiler | ssr | 1/2 (50.0%) | unmeasured | 0/2 | 1 | 0 | 0",
        "",
        "Product | Dialect | Verified rows | Verified native-only | Provisional rows | Provisional native-only",
        "--- | --- | ---: | ---: | ---: | ---:",
        "compiler | vue3-sfc | 4 | 2 | 0 | 0",
        "",
        "### Zero-fallback verdict: does not hold",
        "",
        "Product | Planned | Fallback-free | Legacy fallbacks | Unverified | Non-equivalent | Unmeasured targets | Unmeasured dialects",
        "--- | ---: | ---: | ---: | ---: | ---: | --- | ---",
        "compiler | 4 | 1 | 2 | 0 | 1 | T2/dom, T2/ssr | 1/2",
        "",
        "Fallback-free requires native handling with verified equivalence. Legacy fallbacks are legacy-backed plus unsupported coordinates. A dialect is measured only by verified fixture evidence; provisional labels do not count. Unmeasured coordinates block the verdict.",
        "",
      ].join("\n"),
    );
  });
});

void test("zero fallback holds only when every tier, target and dialect is native and equivalent", () => {
  withRoot((root) => {
    const rows: Row[] = ["dom", "ssr"].map((target) => ({
      id: "compiler/sfc/one",
      target,
      native,
      comparison: { state: "equal" },
    }));
    const artifacts = path.join(root, "artifacts");
    for (const tier of ["T1", "T2"] as const) {
      // Nested like downloaded Actions artifacts: <artifact>/differential/bundle.
      const bundle = path.join(artifacts, `shard-${tier}`, "differential", "bundle");
      writeBundleEntry({
        repoRoot: root,
        bundleDir: bundle,
        tier,
        pack: "cli",
        ...pack(root, rows),
      });
    }
    const entries = readBundles([artifacts]);
    assert.deepEqual(
      entries.map((entry) => `${entry.tier}/${entry.pack}`),
      ["T1/cli", "T2/cli"],
    );
    const merged = path.join(root, "merged");
    copyBundle(entries, merged);
    assert.deepEqual(
      readBundles([merged]).map((entry) => entry.report),
      entries.map((entry) => entry.report),
    );
    const fullPolicy = { ...policy, dialects: ["vue3-sfc"] };
    const verdict = verifyZeroFallback(
      buildCensus(entries, fullPolicy, "c".repeat(64)),
      fullPolicy,
    );
    assert.deepEqual(verdict.blockers, []);
    assert.equal(verdict.holds, true);
    assert.equal(verdict.products.compiler.nativeEquivalent, 4);
  });
});

void test("bundles refuse acceptance their rows contradict, mixed revisions and unknown products", () => {
  withRoot((root) => {
    const bundle = path.join(root, "bundle");
    const dir = writeBundleEntry({
      repoRoot: root,
      bundleDir: bundle,
      tier: "T1",
      pack: "cli",
      ...pack(root, mixedRows),
    });
    const acceptancePath = path.join(dir, "acceptance.json");
    const original = fs.readFileSync(acceptancePath, "utf8");
    const inflated = JSON.parse(original);
    inflated.targets.dom.nativeHandled += 1;
    inflated.targets.dom.unsupported -= 1;
    fs.writeFileSync(acceptancePath, JSON.stringify(inflated));
    assert.throws(() => readBundles([bundle]), /T1\/compiler\/cli\/dom: unsupported/);
    fs.writeFileSync(acceptancePath, original);
    const resultPath = path.join(dir, "result.json");
    const result = JSON.parse(fs.readFileSync(resultPath, "utf8"));
    fs.writeFileSync(resultPath, JSON.stringify({ ...result, rows: result.rows.slice(1) }));
    assert.throws(() => readBundles([bundle]), /missing or extra planned result rows/);
    fs.writeFileSync(resultPath, JSON.stringify(result));

    writeBundleEntry({
      repoRoot: root,
      bundleDir: bundle,
      tier: "T1",
      pack: "other",
      ...pack(root, mixedRows, "d".repeat(40)),
    });
    assert.throws(
      () => buildCensus(readBundles([bundle]), policy, "c".repeat(64)),
      /T1 mixes source revisions/,
    );
    assert.throws(
      () =>
        buildCensus(readBundles([bundle]).slice(0, 1), { ...policy, targets: {} }, "c".repeat(64)),
      /product is not in the deletion-readiness policy/,
    );
  });
});
