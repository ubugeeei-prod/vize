import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertHydratedKnownViolationPaths,
  auditKnownViolationIssues,
  createKnownViolationConsumption,
  validateKnownViolationEntries,
  writeGlyphCorpusPropertyEvidence,
} from "../../tools/fixtures/glyph-corpus.mjs";
import { createWaiverIssueAudit } from "../../tools/fixtures/glyph-corpus-waiver-audit.mjs";

const validEntry = {
  property: "parse-preservation",
  category: "semantic-diff",
  project: "fixture-a",
  path: "src/App.vue",
  reason: "The formatter currently removes an authored semantic token.",
  trackingIssue: 4107,
  expiryCondition: "Remove when the authored fixture passes without a waiver.",
};

const projects = [
  {
    id: "fixture-a",
    fixtureDir: "/fixtures/a",
    hydrated: false,
    vueGlobs: ["src/**/*.vue"],
  },
];

test("known formatter waivers require a precise non-overlapping schema", () => {
  assert.deepEqual(validateKnownViolationEntries([validEntry], projects), [validEntry]);

  for (const [field, value] of [
    ["property", "unknown-property"],
    ["category", "unknown-category"],
    ["project", "unknown-project"],
    ["path", "*"],
    ["trackingIssue", 0],
    ["reason", ""],
    ["expiryCondition", ""],
  ] as const) {
    assert.throws(
      () => validateKnownViolationEntries([{ ...validEntry, [field]: value }], projects),
      new RegExp(String(field)),
    );
  }

  for (const invalidPath of ["src\\App.vue", "src/../App.vue", "src//App.vue", "src/App.ts"]) {
    assert.throws(
      () => validateKnownViolationEntries([{ ...validEntry, path: invalidPath }], projects),
      /exact relative path/,
    );
  }

  assert.throws(
    () => validateKnownViolationEntries([validEntry, { ...validEntry }], projects),
    /duplicate known violation identity/,
  );
  assert.throws(
    () => validateKnownViolationEntries([{ ...validEntry, unexpected: true }], projects),
    /unknown field unexpected/,
  );
  assert.throws(
    () =>
      validateKnownViolationEntries(
        [{ ...validEntry, property: "lint-agreement", rule: undefined }],
        projects,
      ),
    /lint-agreement.*rule/,
  );
  assert.throws(
    () => validateKnownViolationEntries([{ ...validEntry, rule: "vue/no-v-html" }], projects),
    /rule.*lint-agreement/,
  );
});

test("hydrated waiver paths must exist inside the registered Vue corpus", () => {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-waiver-path-"));
  const project = {
    ...projects[0],
    fixtureDir,
    hydrated: true,
  };
  try {
    assert.throws(
      () => assertHydratedKnownViolationPaths([validEntry], [project]),
      /missing waived fixture path/,
    );
    fs.mkdirSync(path.join(fixtureDir, "src"), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, "src", "App.vue"), "<template><p /></template>\n");
    assert.doesNotThrow(() => assertHydratedKnownViolationPaths([validEntry], [project]));

    const outsideGlobs = [{ ...validEntry, path: "other/Outside.vue" }];
    fs.mkdirSync(path.join(fixtureDir, "other"), { recursive: true });
    fs.writeFileSync(path.join(fixtureDir, "other", "Outside.vue"), "<template><p /></template>\n");
    assert.throws(
      () => assertHydratedKnownViolationPaths(outsideGlobs, [project]),
      /outside registered Vue inputs/,
    );
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
  }
});

test("every hydrated waiver is consumed exactly once with the same category", () => {
  const tracker = createKnownViolationConsumption([validEntry]);
  assert.throws(() => tracker.assertAllConsumed(new Set(["fixture-a"])), /stale unconsumed waiver/);
  assert.throws(
    () => tracker.consume("fixture-a", "src/App.vue", null, "baseline-unusable"),
    /category mismatch/,
  );
  assert.deepEqual(tracker.consume("fixture-a", "src/App.vue", null, "semantic-diff"), validEntry);
  assert.doesNotThrow(() => tracker.assertAllConsumed(new Set(["fixture-a"])));
  assert.throws(
    () => tracker.consume("fixture-a", "src/App.vue", null, "semantic-diff"),
    /consumed more than once/,
  );

  const untouched = createKnownViolationConsumption([validEntry]);
  assert.doesNotThrow(() => untouched.assertAllConsumed(new Set(["another-project"])));
  assert.equal(untouched.consume("fixture-a", "src/Other.vue", null, "semantic-diff"), null);
});

test("release evidence records each live owner and its exact waiver count", async () => {
  const requests: string[] = [];
  const artifact = await createWaiverIssueAudit(
    [validEntry, { ...validEntry, path: "src/Other.vue" }],
    {
      GITHUB_API_URL: "https://github.example/api/v3",
      GITHUB_REPOSITORY: "owner/repo",
      GITHUB_SHA: "0123456789abcdef",
      GITHUB_TOKEN: "secret",
    },
    async (url: string) => {
      requests.push(url);
      return {
        ok: true,
        json: async () => ({
          number: 4107,
          state: "open",
          title: "strict owner",
          html_url: "https://github.example/owner/repo/issues/4107",
          updated_at: "2026-08-10T00:00:00Z",
        }),
      } as Response;
    },
  );
  assert.deepEqual(requests, ["https://github.example/api/v3/repos/owner/repo/issues/4107"]);
  assert.equal(artifact.schema, "vize.glyphCorpusWaiverIssueAudit");
  assert.equal(artifact.version, 1);
  assert.equal(artifact.repository, "owner/repo");
  assert.equal(artifact.sourceCommit, "0123456789abcdef");
  assert.equal(artifact.waiverCount, 2);
  assert.deepEqual(artifact.issues, [
    {
      number: 4107,
      state: "OPEN",
      title: "strict owner",
      url: "https://github.example/owner/repo/issues/4107",
      updatedAt: "2026-08-10T00:00:00Z",
      waiverCount: 2,
    },
  ]);
  assert.match(artifact.generatedAt, /^\d{4}-\d{2}-\d{2}T/);
});

test("tracking Issue evidence is deduplicated and fails closed on closed Issues", async () => {
  const calls: number[] = [];
  const open = await auditKnownViolationIssues(
    [validEntry, { ...validEntry, path: "src/Other.vue" }],
    async (number: number) => {
      calls.push(number);
      return { number, state: "OPEN", title: "open owner", url: `https://example.test/${number}` };
    },
  );
  assert.deepEqual(calls, [4107]);
  assert.deepEqual(open, [
    {
      number: 4107,
      state: "OPEN",
      title: "open owner",
      url: "https://example.test/4107",
    },
  ]);

  await assert.rejects(
    () =>
      auditKnownViolationIssues([validEntry], async (number: number) => ({
        number,
        state: "CLOSED",
        title: "fixed owner",
        url: `https://example.test/${number}`,
      })),
    /tracking Issue #4107 is CLOSED/,
  );
});

test("formatter property artifacts retain waived and unwaived difference details", () => {
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-glyph-evidence-"));
  try {
    const output = writeGlyphCorpusPropertyEvidence(
      "parse-preservation",
      {
        projectIds: ["fixture-b", "fixture-a"],
        counters: { files: 7, skipped: 1 },
        violations: [{ project: "fixture-b", file: "src/B.vue", detail: "block changed" }],
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
      violationCount: 1,
      waiverValidationError: null,
    });
    assert.equal(artifact.waivedDifferences[0].detail, "full comparator evidence");
    assert.equal(artifact.violations[0].detail, "block changed");
  } finally {
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
