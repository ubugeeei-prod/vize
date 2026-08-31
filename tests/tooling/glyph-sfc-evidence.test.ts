import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  createGlyphSfcEquivalenceEvidence,
  validateGlyphSfcEquivalenceEvidence,
  writeGlyphSfcEquivalenceEvidence,
} from "../../legacy-tools/fixtures/glyph-sfc-evidence.mjs";
import {
  glyphSfcEvidenceInput as input,
  hash,
  resign,
} from "./support/glyph-sfc-evidence-fixture.ts";

test("glyph SFC evidence binds per-file dialect, compiler, hashes, and verdict", () => {
  const artifact = createGlyphSfcEquivalenceEvidence(input());
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
  assert.doesNotThrow(() => validateGlyphSfcEquivalenceEvidence(artifact, input().expectedFiles));
});

test("glyph SFC evidence ordering is locale-independent", () => {
  const base = input();
  const extra = {
    ...base.files[0],
    project: "z-project",
    path: "src/Z.vue",
  };
  const unicode = {
    ...base.files[0],
    project: "ä-project",
    path: "src/A.vue",
  };
  const artifact = createGlyphSfcEquivalenceEvidence({
    ...base,
    files: [unicode, extra],
    expectedFiles: [
      { ...base.expectedFiles[0], project: extra.project, path: extra.path },
      { ...base.expectedFiles[0], project: unicode.project, path: unicode.path },
    ],
  });
  assert.deepEqual(
    artifact.files.map(({ project }) => project),
    ["z-project", "ä-project"],
  );
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
  for (const [mutation, expected] of [
    [
      { files: [{ ...artifact.files[0], originalSha256: "bad" }] },
      /originalSha256 must be a sha256/,
    ],
    [
      { files: [{ ...artifact.files[0], differences: ["hidden corruption"] }] },
      /equivalent glyph SFC evidence is inconsistent/,
    ],
    [
      { files: [{ ...artifact.files[0], waiver: { issue: 4108 } }] },
      /glyph SFC dialect evidence cannot be waived/,
    ],
    [
      { files: [{ ...artifact.files[0], revision: "not-a-commit" }] },
      /fixture revision must be an exact commit/,
    ],
    [{ files: [{ ...artifact.files[0], routeId: "" }] }, /invalid routeId/],
    [{ baselines: forgeBaseline({ dialect: "4" }) }, /baseline vue2\.6 has an invalid dialect/],
    [
      { baselines: forgeBaseline({ package: "forged-compiler" }) },
      /vue2\.6 package violates the pinned contract/,
    ],
    [
      { baselines: forgeBaseline({ version: "0.0.0" }) },
      /vue2\.6 version violates the pinned contract/,
    ],
    [
      { baselines: forgeBaseline({ normalization: "" }) },
      /vue2\.6 normalization must be non-empty/,
    ],
    [
      { baselines: forgeBaseline({ options: { whitespace: "condense" } }) },
      /vue2\.6 options violate the pinned contract/,
    ],
    [{ baselines: forgeBaseline({ entrySha256: "bad" }) }, /vue2\.6 entry sha256 must be a sha256/],
    [
      { baselines: artifact.baselines.slice(1) },
      /missing baseline contract: unsupported-vue-0\.10/,
    ],
    [{ sourceCommit: null }, /sourceCommit must be an exact commit/],
  ] as const) {
    assert.throws(
      () => validateGlyphSfcEquivalenceEvidence(resign({ ...artifact, ...mutation })),
      expected,
    );
  }
  assert.throws(
    () => validateGlyphSfcEquivalenceEvidence({ ...artifact, sha256: hash("forged") }),
    /artifact digest mismatch/,
  );
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
    differences: ["template-compile: crash"],
    failure: { side: "formatted", stage: "template-compile", message: "crash" },
  };
  const harnessFailure = {
    ...originalFailure,
    reasonCode: "comparison-harness-unusable",
    differences: ["comparison-harness: crash"],
    failure: { side: "harness", stage: "comparison-harness", message: "crash" },
  };
  assert.equal(
    createGlyphSfcEquivalenceEvidence({ ...base, files: [originalFailure] }).files[0].verdict,
    "baseline-unusable",
  );
  assert.equal(
    createGlyphSfcEquivalenceEvidence({ ...base, files: [formattedFailure] }).files[0].verdict,
    "semantic-diff",
  );
  assert.equal(
    createGlyphSfcEquivalenceEvidence({ ...base, files: [harnessFailure] }).files[0].reasonCode,
    "comparison-harness-unusable",
  );

  const originalArtifact = createGlyphSfcEquivalenceEvidence({ ...base, files: [originalFailure] });
  const formattedArtifact = createGlyphSfcEquivalenceEvidence({
    ...base,
    files: [formattedFailure],
  });
  assert.throws(
    () =>
      validateGlyphSfcEquivalenceEvidence(
        resign({
          ...originalArtifact,
          files: [
            {
              ...originalArtifact.files[0],
              failure: { ...originalFailure.failure, side: "formatted" },
            },
          ],
        }),
      ),
    /original-baseline-unusable ownership is invalid/,
  );
  assert.throws(
    () =>
      validateGlyphSfcEquivalenceEvidence(
        resign({
          ...formattedArtifact,
          files: [
            {
              ...formattedArtifact.files[0],
              failure: { ...formattedFailure.failure, side: "original" },
            },
          ],
        }),
      ),
    /formatted-baseline-unusable ownership is invalid/,
  );
  assert.throws(
    () =>
      validateGlyphSfcEquivalenceEvidence(
        resign({
          ...originalArtifact,
          files: [
            {
              ...originalArtifact.files[0],
              failure: { ...originalFailure.failure, stage: "comparison-harness" },
              differences: ["comparison-harness: crash"],
            },
          ],
        }),
      ),
    /failure stage ownership is invalid/,
  );
});
