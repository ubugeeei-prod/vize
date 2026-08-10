import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import {
  createGlyphSfcEquivalenceEvidence,
  validateGlyphSfcEquivalenceEvidence,
} from "../../tools/fixtures/glyph-sfc-evidence.mjs";

const hash = (value: string): string => createHash("sha256").update(value).digest("hex");

function input() {
  const semantic = hash("semantic");
  return {
    sourceCommit: "a".repeat(40),
    formatter: { version: "0.346.0", binarySha256: hash("vize") },
    waiverValidationError: null,
    availableBaselines: [
      {
        id: "vue2.7",
        dialect: "2.7",
        package: "@vue/compiler-sfc",
        version: "2.7.16",
        entrySha256: hash("compiler-2.7"),
        normalization: "vue2-render-v1",
        options: { isProduction: true },
      },
    ],
    expectedFiles: [
      {
        project: "gogocode",
        revision: "b".repeat(40),
        path: "src/App.vue",
        routeId: "vue2",
        dialect: "2",
        baselineId: "vue2.6",
      },
    ],
    files: [
      {
        project: "gogocode",
        revision: "b".repeat(40),
        path: "src/App.vue",
        routeId: "vue2",
        dialect: "2",
        baselineId: "vue2.6",
        originalSha256: hash("original"),
        formattedSha256: hash("formatted"),
        beforeSemanticSha256: semantic,
        afterSemanticSha256: semantic,
        verdict: "equivalent",
        reasonCode: null,
        differences: [],
        failure: null,
        waiver: null,
        baseline: {
          id: "vue2.6",
          dialect: "2",
          package: "vue-template-compiler",
          version: "2.6.14",
          entrySha256: hash("compiler"),
          normalization: "vue2-render-v1",
          options: { comments: true },
        },
      },
    ],
  };
}

test("glyph SFC evidence binds per-file dialect, compiler, hashes, and verdict", () => {
  const artifact = createGlyphSfcEquivalenceEvidence(input());
  assert.equal(artifact.files.length, 1);
  assert.equal(artifact.files[0].dialect, "2");
  assert.deepEqual(
    artifact.baselines.map(({ id, version }) => ({ id, version })),
    [
      { id: "vue2.6", version: "2.6.14" },
      { id: "vue2.7", version: "2.7.16" },
    ],
  );
  assert.deepEqual(artifact.summary, {
    fileCount: 1,
    verdictCounts: {
      equivalent: 1,
      "semantic-diff": 0,
      "baseline-unusable": 0,
      "oracle-unavailable": 0,
    },
    waivedDifferenceCount: 0,
    waiverValidationError: null,
  });
  assert.equal(artifact.sha256.length, 64);
  assert.doesNotThrow(() => validateGlyphSfcEquivalenceEvidence(artifact, input().expectedFiles));
});

test("glyph SFC evidence rejects missing, duplicate, and mismatched files", () => {
  const artifact = createGlyphSfcEquivalenceEvidence(input());
  assert.throws(
    () => validateGlyphSfcEquivalenceEvidence({ ...artifact, files: [] }, input().expectedFiles),
    /missing glyph SFC evidence/,
  );
  assert.throws(
    () =>
      validateGlyphSfcEquivalenceEvidence(
        { ...artifact, files: [artifact.files[0], artifact.files[0]] },
        input().expectedFiles,
      ),
    /duplicate glyph SFC evidence/,
  );
  assert.throws(
    () =>
      validateGlyphSfcEquivalenceEvidence({
        ...artifact,
        files: [{ ...artifact.files[0], dialect: "3" }],
      }),
    /baseline\/dialect mismatch/,
  );
});

test("glyph SFC evidence fails closed on forged verdicts and provenance", () => {
  const artifact = createGlyphSfcEquivalenceEvidence(input());
  for (const mutation of [
    { files: [{ ...artifact.files[0], originalSha256: "bad" }] },
    { files: [{ ...artifact.files[0], differences: ["hidden corruption"] }] },
    { files: [{ ...artifact.files[0], revision: "not-a-commit" }] },
    { files: [{ ...artifact.files[0], routeId: "" }] },
    { baselines: [{ ...artifact.baselines[0], dialect: "4" }] },
    { baselines: [{ ...artifact.baselines[0], normalization: "" }] },
    { baselines: [{ ...artifact.baselines[0], options: null }] },
    { sourceCommit: null },
    { sha256: hash("forged") },
  ]) {
    assert.throws(() => validateGlyphSfcEquivalenceEvidence({ ...artifact, ...mutation }));
  }
});

test("glyph SFC evidence is bound to the registry-selected route", () => {
  const base = input();
  const artifact = createGlyphSfcEquivalenceEvidence(base);
  for (const field of ["revision", "routeId", "dialect", "baselineId"] as const) {
    const expectedFiles = [{ ...base.expectedFiles[0], [field]: "forged" }];
    assert.throws(
      () => validateGlyphSfcEquivalenceEvidence(artifact, expectedFiles),
      new RegExp(`${field} mismatch`),
    );
  }
});

test("baseline crashes and formatter-caused crashes keep distinct ownership", () => {
  const base = input();
  const originalFailure = {
    ...base.files[0],
    verdict: "baseline-unusable",
    reasonCode: "original-baseline-unusable",
    differences: ["sfc-parse: crash"],
    failure: { side: "original", stage: "sfc-parse", message: "crash" },
    beforeSemanticSha256: null,
    afterSemanticSha256: null,
  };
  const formattedFailure = {
    ...originalFailure,
    verdict: "semantic-diff",
    reasonCode: "formatted-baseline-unusable",
    failure: { side: "formatted", stage: "template-compile", message: "crash" },
  };
  assert.equal(
    createGlyphSfcEquivalenceEvidence({ ...base, files: [originalFailure] }).files[0].verdict,
    "baseline-unusable",
  );
  assert.equal(
    createGlyphSfcEquivalenceEvidence({ ...base, files: [formattedFailure] }).files[0].verdict,
    "semantic-diff",
  );
});
