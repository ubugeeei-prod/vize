import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  classifyNativeRow,
  loadProductManifest,
  readPinnedArtifact,
  runPlannedCases,
  sha256,
  validateResultEnvelope,
} from "../differential/harness.mjs";

function withManifest(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-product-harness-"));
  const sourceRevision = "a".repeat(40);
  const manifest = {
    schema: "vize.differential.manifest",
    version: 1,
    product: "compiler",
    baseRevision: sourceRevision,
    cases: [{ id: "compiler/sfc/basic", state: "active", targets: ["dom", "ssr"] }],
  };
  try {
    fs.writeFileSync(path.join(root, "input.vue"), "<template>é</template>\r\n");
    const manifestPath = path.join(root, "manifest.json");
    fs.writeFileSync(manifestPath, JSON.stringify(manifest));
    run({ root, manifest, manifestPath, sourceRevision });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

const row = (target) => ({
  id: "compiler/sfc/basic",
  target,
  legacy: { state: "completed" },
  native: { state: "unsupported", reason: "no native product path" },
  comparison: { state: "not-compared" },
});

void test("product adapter retains every planned case and target in one result envelope", () => {
  withManifest(({ manifestPath, sourceRevision }) => {
    const loaded = loadProductManifest(manifestPath, "compiler");
    const rows = runPlannedCases(loaded, {
      runCase: (fixture) => fixture.targets.map(row),
    });
    const report = {
      schema: "vize.differential.result",
      version: 1,
      product: "compiler",
      manifestSha256: loaded.manifestSha256,
      sourceRevision,
      rows,
      summary: { plannedCases: 1 },
    };
    validateResultEnvelope(loaded, report, sourceRevision);
    assert.throws(
      () => validateResultEnvelope(loaded, { ...report, rows: rows.slice(0, 1) }, sourceRevision),
      /missing or extra/,
    );
    assert.throws(
      () =>
        validateResultEnvelope(
          loaded,
          { ...report, rows: [row("dom"), row("dom")] },
          sourceRevision,
        ),
      /duplicate result row/,
    );
    assert.throws(
      () =>
        validateResultEnvelope(
          loaded,
          { ...report, rows: [row("dom"), row("vapor")] },
          sourceRevision,
        ),
      /unplanned result target/,
    );
    assert.throws(() =>
      validateResultEnvelope(loaded, { ...report, manifestSha256: "0".repeat(64) }, sourceRevision),
    );
  });
});

void test("manifest product and pinned input bytes cannot drift", () => {
  withManifest(({ root, manifest, manifestPath }) => {
    assert.throws(() => loadProductManifest(manifestPath, "linter"), /no registered adapter/);
    manifest.cases.push({ ...manifest.cases[0] });
    fs.writeFileSync(manifestPath, JSON.stringify(manifest));
    assert.throws(() => loadProductManifest(manifestPath, "compiler"), /duplicate planned case/);
    const file = {
      path: "input.vue",
      sha256: sha256(fs.readFileSync(path.join(root, "input.vue"))),
    };
    assert.equal(readPinnedArtifact(root, file).toString(), "<template>é</template>\r\n");
    fs.appendFileSync(path.join(root, "input.vue"), " ");
    assert.throws(() => readPinnedArtifact(root, file), /SHA256 mismatch/);
  });
});

void test("native credit requires a verified whole-product transcript and observation", () => {
  const sourceRevision = "a".repeat(40);
  const buildReceiptSha256 = "b".repeat(64);
  const native = {
    state: "completed",
    observation: { outputSha256: "c".repeat(64) },
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
  const options = {
    sourceRevision,
    buildReceiptSha256,
    requiredStages: ["sfc_parse", "dom_emit"],
    verifyObservation: (observation) => observation?.outputSha256 === "c".repeat(64),
  };
  assert.equal(classifyNativeRow({ native }, options), "native-handled");
  assert.equal(classifyNativeRow({ native: { state: "unsupported" } }, options), "unsupported");
  assert.equal(
    classifyNativeRow({ native }, { ...options, sourceRevision: "d".repeat(40) }),
    "unverified",
  );
  assert.equal(
    classifyNativeRow({ native }, { ...options, verifyObservation: () => false }),
    "unverified",
  );
  const legacy = structuredClone(native);
  legacy.provenance.contributions[0].factOrigin = "legacy";
  assert.equal(classifyNativeRow({ native: legacy }, options), "legacy-backed");
  const missing = structuredClone(native);
  missing.provenance.contributions.pop();
  assert.equal(classifyNativeRow({ native: missing }, options), "unverified");
  const extra = structuredClone(native);
  extra.provenance.contributions.push({
    stage: "unexpected",
    implementation: "native",
    factOrigin: "native",
    fallback: false,
  });
  assert.equal(classifyNativeRow({ native: extra }, options), "unverified");
  const fallback = structuredClone(native);
  fallback.provenance.contributions[1].fallback = true;
  assert.equal(classifyNativeRow({ native: fallback }, options), "legacy-backed");
});
