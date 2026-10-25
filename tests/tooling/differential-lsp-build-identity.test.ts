import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { validateLspReport } from "../differential/lsp-report.ts";
import { runLspPack } from "../differential/lsp.ts";
import type { BuildIdentity } from "../differential/lsp-types.ts";
import { referenceReport } from "./support/lsp/differential-reference.ts";
import { root } from "./support/lsp/paths.ts";

const manifestPath = path.join(root, "tests/_fixtures/differential/lsp/manifest.json");
const loaded = loadLspManifest(manifestPath);
const build = {
  sourceRevision: loaded.manifest.baseRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.474.0-dev",
};

test("claimed LSP build receipts require every expected executable identity field", () => {
  // Synthetic raw frames exercise admission laws, not product/native credit.
  const report = referenceReport(loaded, build);
  validateLspReport(loaded, report, build);
  for (const field of ["binaryPath", "binarySha256", "cliVersion"] as const) {
    const incomplete: BuildIdentity = { ...build };
    delete incomplete[field];
    assert.throws(
      () => validateLspReport(loaded, report, incomplete),
      new RegExp(`source receipt requires expected ${field}`),
    );
  }
});

test("receipt-less LSP launch failures retain source-only expected identity", async () => {
  const report = await runLspPack({
    manifestPath,
    sourceRevision: build.sourceRevision,
    repoRoot: root,
  });
  assert.equal(report.buildReceipt, null);
  assert.equal(report.binary, null);
  assert.equal(report.summary.legacyFailures, loaded.cases.length);
  assert.equal(report.summary.legacyMatches, 0);
  validateLspReport(loaded, report, { sourceRevision: build.sourceRevision });
});
