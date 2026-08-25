import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  selectGlyphCorpusProjects,
  writeGlyphCorpusPropertyEvidence,
} from "../../tools/fixtures/glyph-corpus.mjs";

const validEntry = {
  property: "parse-preservation",
  category: "semantic-diff",
  project: "fixture-a",
  path: "src/App.vue",
  reason: "The formatter currently removes an authored semantic token.",
  trackingIssue: 4107,
  expiryCondition: "Remove when the authored fixture passes without a waiver.",
};

test("formatter property artifacts retain waived and unwaived difference details", () => {
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-evidence-"));
  try {
    const output = writeGlyphCorpusPropertyEvidence(
      "parse-preservation",
      {
        projectIds: ["fixture-b", "fixture-a"],
        counters: { files: 7, skipped: 1 },
        violations: [{ project: "fixture-b", file: "src/B.vue", detail: "block changed" }],
        baselineUnusable: [
          {
            project: "fixture-c",
            file: "src/Baseline.vue",
            detail: "pristine Vue baseline failed",
          },
        ],
        waivedViolations: [
          {
            project: "fixture-a",
            file: "src/App.vue",
            detail: "full comparator evidence",
            waiver: validEntry,
          },
        ],
        waiverValidationError: null,
      },
      reportDir,
    );
    assert.equal(output, path.join(reportDir, "glyph-parse-preservation.json"));
    const artifact = JSON.parse(fs.readFileSync(output!, "utf8"));
    assert.equal(artifact.schema, "vize.glyphCorpusPropertyEvidence");
    assert.equal(artifact.version, 1);
    assert.deepEqual(artifact.projectIds, ["fixture-a", "fixture-b"]);
    assert.deepEqual(artifact.summary, {
      cleanFileCount: 7,
      waivedDifferenceCount: 1,
      baselineUnusableCount: 1,
      violationCount: 1,
      waiverValidationError: null,
    });
    assert.equal(artifact.waivedDifferences[0].detail, "full comparator evidence");
    assert.equal(artifact.baselineUnusable[0].detail, "pristine Vue baseline failed");
    assert.equal(artifact.violations[0].detail, "block changed");
  } finally {
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});

test("glyph corpus shard selection is bound to manifest project ids before hydration", () => {
  const projects = [
    {
      id: "other",
      coverage: ["formatter"],
      expectedVueFileCount: 1,
      fixturePath: "tests/_fixtures/_git/other",
    },
    {
      id: "primevue",
      coverage: ["formatter"],
      expectedVueFileCount: 1,
      fixturePath: "tests/_fixtures/_git/primevue",
    },
    {
      id: "primevue-volt",
      coverage: ["formatter"],
      expectedVueFileCount: 1,
      fixturePath: "tests/_fixtures/_git/primevue",
    },
    {
      id: "shared-non-formatter",
      coverage: ["compiler"],
      expectedVueFileCount: 1,
      fixturePath: "tests/_fixtures/_git/primevue",
    },
  ];

  const selected = selectGlyphCorpusProjects(projects, {
    FIXTURE_SHARD_INDEX: "1",
    FIXTURE_SHARD_COUNT: "2",
  });
  assert.deepEqual(
    selected.map((project: { id: string }) => project.id),
    ["primevue"],
  );
  assert.throws(
    () => selectGlyphCorpusProjects(projects, { FIXTURE_SHARD_COUNT: "2" }),
    /must be set together/,
  );
  assert.throws(
    () =>
      selectGlyphCorpusProjects(projects, {
        FIXTURE_SHARD_INDEX: "2",
        FIXTURE_SHARD_COUNT: "2",
      }),
    /must be less than/,
  );
});
