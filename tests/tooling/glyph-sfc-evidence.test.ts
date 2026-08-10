import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  createGlyphSfcEquivalenceEvidence,
  validateGlyphSfcEquivalenceEvidence,
  writeGlyphSfcEquivalenceEvidence,
} from "../../tools/fixtures/glyph-sfc-evidence.mjs";

const hash = (value: string): string => createHash("sha256").update(value).digest("hex");

function input() {
  const semantic = hash("semantic");
  return {
    sourceCommit: "a".repeat(40),
    formatter: { version: "0.346.0", binarySha256: hash("vize") },
    waiverValidationError: null,
    availableBaselines: [
      ...["0.10", "0.11", "1"].map((dialect) => ({
        id: `unsupported-vue-${dialect}`,
        dialect,
        package: null,
        version: null,
        entrySha256: null,
        normalization: "unavailable",
        options: {},
      })),
      {
        id: "vue2.7",
        dialect: "2.7",
        package: "@vue/compiler-sfc",
        version: "2.7.16",
        entrySha256: hash("compiler-2.7"),
        normalization: "vue2-render-v1",
        options: {
          parse: { pad: false },
          compile: {
            isProduction: true,
            prettify: false,
            compilerOptions: {
              comments: true,
              outputSourceRange: true,
              whitespace: "preserve",
            },
          },
        },
      },
      {
        id: "vue3",
        dialect: "3",
        package: "@vue/compiler-sfc",
        version: "3.6.0-beta.10",
        entrySha256: hash("compiler-3"),
        normalization: "vue3-template-ast-v1",
        options: { sourceMap: false },
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
          options: {
            parse: { pad: false },
            compile: { comments: true, outputSourceRange: true, whitespace: "preserve" },
          },
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
    artifact.baselines.map(({ id }) => id),
    [
      "unsupported-vue-0.10",
      "unsupported-vue-0.11",
      "unsupported-vue-1",
      "vue2.6",
      "vue2.7",
      "vue3",
    ],
  );
  assert.deepEqual(artifact.summary, {
    fileCount: 1,
    verdictCounts: {
      equivalent: 1,
      "semantic-diff": 0,
      "baseline-unusable": 0,
    },
    waivedDifferenceCount: 0,
    waiverValidationError: null,
  });
  assert.equal(artifact.sha256.length, 64);
  assert.doesNotThrow(() => validateGlyphSfcEquivalenceEvidence(artifact, input().expectedFiles));
});

test("dialect evidence cannot overwrite the corpus property artifact", () => {
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-sfc-evidence-"));
  try {
    const output = writeGlyphSfcEquivalenceEvidence(input(), reportDir);
    assert.equal(output, path.join(reportDir, "glyph-sfc-dialect-equivalence.json"));
    assert.equal(JSON.parse(fs.readFileSync(output, "utf8")).files.length, 1);
  } finally {
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
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
  const forgeBaseline = (changes: Record<string, unknown>) =>
    artifact.baselines.map((baseline) =>
      baseline.id === "vue2.6" ? { ...baseline, ...changes } : baseline,
    );
  for (const mutation of [
    { files: [{ ...artifact.files[0], originalSha256: "bad" }] },
    { files: [{ ...artifact.files[0], differences: ["hidden corruption"] }] },
    { files: [{ ...artifact.files[0], revision: "not-a-commit" }] },
    { files: [{ ...artifact.files[0], routeId: "" }] },
    { baselines: forgeBaseline({ dialect: "4" }) },
    { baselines: forgeBaseline({ package: "forged-compiler" }) },
    { baselines: forgeBaseline({ version: "0.0.0" }) },
    { baselines: forgeBaseline({ normalization: "" }) },
    { baselines: forgeBaseline({ options: { whitespace: "condense" } }) },
    { baselines: artifact.baselines.slice(1) },
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
