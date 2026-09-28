import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  renderNativeAcceptanceMarkdown,
  summarizeNativeAcceptance,
} from "../differential/acceptance-rates.mjs";
import { loadProductManifest } from "../differential/harness.mjs";

const sourceRevision = "a".repeat(40);
const buildReceiptSha256 = "b".repeat(64);
const outputSha256 = "c".repeat(64);

function withFixture(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-acceptance-rates-"));
  const manifestPath = path.join(root, "manifest.json");
  fs.writeFileSync(
    manifestPath,
    JSON.stringify({
      schema: "vize.differential.manifest",
      version: 1,
      product: "compiler",
      baseRevision: sourceRevision,
      cases: [
        { id: "compiler/sfc/one", state: "active", targets: ["dom", "ssr"] },
        { id: "compiler/sfc/two", state: "active", targets: ["dom", "ssr"] },
      ],
    }),
  );
  try {
    const loaded = loadProductManifest(manifestPath, "compiler");
    const native = {
      state: "completed",
      observation: { outputSha256 },
      provenance: {
        scope: "whole-product",
        sourceRevision,
        buildReceiptSha256,
        contributions: [
          { stage: "sfc_parse", implementation: "native", factOrigin: "native", fallback: false },
          { stage: "dom_emit", implementation: "native", factOrigin: "native", fallback: false },
        ],
      },
    };
    const rows = [
      {
        id: "compiler/sfc/one",
        target: "dom",
        legacy: { state: "completed" },
        native,
        comparison: { state: "equal" },
      },
      {
        id: "compiler/sfc/one",
        target: "ssr",
        legacy: { state: "completed" },
        native: {
          ...native,
          provenance: {
            ...native.provenance,
            contributions: [
              {
                stage: "sfc_parse",
                implementation: "native",
                factOrigin: "legacy",
                fallback: false,
              },
              {
                stage: "ssr_emit",
                implementation: "native",
                factOrigin: "native",
                fallback: false,
              },
            ],
          },
        },
        comparison: { state: "equal" },
      },
      {
        id: "compiler/sfc/two",
        target: "dom",
        legacy: { state: "completed" },
        native: { state: "unsupported" },
        comparison: { state: "not-compared" },
      },
      {
        id: "compiler/sfc/two",
        target: "ssr",
        legacy: { state: "completed" },
        native: { state: "failed" },
        comparison: { state: "not-compared" },
      },
    ];
    const report = {
      schema: "vize.differential.result",
      version: 1,
      product: "compiler",
      manifestSha256: loaded.manifestSha256,
      sourceRevision,
      rows,
      summary: { plannedCases: 2 },
    };
    const options = {
      sourceRevision,
      buildReceiptSha256,
      requiredStages: (_row, target) => ["sfc_parse", `${target}_emit`],
      verifyObservation: (observation) => observation?.outputSha256 === outputSha256,
      verifyComparison: (row) => row.comparison.state === "equal" && row.target === "dom",
    };
    run({ loaded, report, options });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

void test("native-only rate counts every planned target and requires verified parity", () => {
  withFixture(({ loaded, report, options }) => {
    const summary = summarizeNativeAcceptance(loaded, report, options);
    assert.equal(summary.plannedCases, 2);
    assert.deepEqual(summary.total, {
      planned: 4,
      nativeHandled: 1,
      nativeEquivalent: 1,
      unsupported: 1,
      legacyBacked: 1,
      unverified: 1,
    });
    assert.deepEqual(summary.targets.dom, {
      planned: 2,
      nativeHandled: 1,
      nativeEquivalent: 1,
      unsupported: 1,
      legacyBacked: 0,
      unverified: 0,
    });
    assert.deepEqual(summary.targets.ssr, {
      planned: 2,
      nativeHandled: 0,
      nativeEquivalent: 0,
      unsupported: 0,
      legacyBacked: 1,
      unverified: 1,
    });
    const unverifiedParity = summarizeNativeAcceptance(loaded, report, {
      ...options,
      verifyComparison: () => false,
    });
    assert.equal(unverifiedParity.total.nativeHandled, 1);
    assert.equal(unverifiedParity.total.nativeEquivalent, 0);
    assert.match(renderNativeAcceptanceMarkdown([summary]), /compiler \| dom \| 1\/2 \(50\.0%\)/);
    assert.match(renderNativeAcceptanceMarkdown([summary]), /Unregistered products.*unmeasured/);
  });
});

void test("rate rejects an unplanned coordinate or wrong source revision", () => {
  withFixture(({ loaded, report, options }) => {
    assert.throws(
      () => summarizeNativeAcceptance(loaded, { ...report, rows: report.rows.slice(1) }, options),
      /missing or extra planned result rows/,
    );
    assert.throws(
      () =>
        summarizeNativeAcceptance(loaded, report, { ...options, sourceRevision: "d".repeat(40) }),
      /unexpected actual source revision/,
    );
    const unverified = summarizeNativeAcceptance(loaded, report, {
      ...options,
      verifyObservation: () => false,
    });
    assert.equal(unverified.total.nativeHandled, 0);
    assert.equal(unverified.total.unverified, 2);
    const staleBuild = summarizeNativeAcceptance(loaded, report, {
      ...options,
      buildReceiptSha256: "d".repeat(64),
    });
    assert.equal(staleBuild.total.nativeHandled, 0);
    assert.equal(staleBuild.total.legacyBacked, 0);
  });
});
