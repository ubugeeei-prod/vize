import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  COHORT,
  COHORTS,
  JSX_COHORT,
  DISPOSITIONS,
  readCohort,
  readDispositions,
  validateHistoryReviews,
} from "../../tools/commands/fixtures/compiler-fix-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const indexPath = path.join(root, "docs/davinci/plan/compiler-fix-history.tsv");
const receiptPath = path.join(root, "docs/davinci/plan/compiler-fix-history-index.json");
const digest = (bytes: string) => createHash("sha256").update(bytes).digest("hex");

void test("bounded compiler history reviews stay tied to complete differential fixtures", () => {
  const reviewed = validateHistoryReviews(root);
  // Counts derive from the review ledger; the floor only stops shrinkage.
  assert(reviewed.reviews.size >= 16);

  const index = fs.readFileSync(indexPath, "utf8");
  const [header, ...rows] = index.trimEnd().split("\n");
  const columns = header.split("\t");
  const sha = columns.indexOf("fix_sha");
  const status = columns.indexOf("review");
  assert(sha >= 0 && status >= 0);
  const indexed = new Map(rows.map((line) => line.split("\t")).map((row) => [row[sha], row]));
  const reviewedStatus = "fixture-linked;bounded-semantic-scope-reviewed;whole-fix-review-pending";
  for (const commit of reviewed.reviews.keys())
    assert.equal(indexed.get(commit)?.[status], reviewedStatus, commit);
  assert.equal(
    rows.filter((line) => line.split("\t")[status] === reviewedStatus).length,
    reviewed.reviews.size,
  );

  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  assert.equal(receipt.version, 4);
  assert.equal(receipt.nonMergeFixSubjects, rows.length);
  assert.equal(receipt.boundedReviewedFixtureScopes, reviewed.reviews.size);
  assert.equal(receipt.wholeFixReviewedSubjects, 0);
  assert.equal(receipt.wholeFixReviewPendingSubjects, rows.length);
  assert.equal(receipt.semanticDenominator, null);
  assert.equal(receipt.nativeAcceptance, 0);
  assert.equal(receipt.index.sha256, digest(index));
  assert.deepEqual(receipt.reviewLedger, {
    path: reviewed.path,
    sha256: reviewed.sha256,
    differentialManifests: reviewed.manifests,
  });
});

void test("current-API compiler oracles and dispositions match the history index", () => {
  const index = fs.readFileSync(indexPath, "utf8");
  const [header, ...rows] = index.trimEnd().split("\n");
  const columns = header.split("\t");
  const sha = columns.indexOf("fix_sha");
  const status = columns.indexOf("review");
  const references = columns.indexOf("fixture_references");
  const indexed = new Map(rows.map((line) => line.split("\t")).map((row) => [row[sha], row]));
  const cohort = readCohort(root);
  const jsxCohort = readCohort(root, JSX_COHORT);
  const pinned = new Set([...cohort, ...jsxCohort].map((fixture) => fixture.history));
  for (const cohortPath of COHORTS)
    for (const fixture of readCohort(root, cohortPath)) {
      const row = indexed.get(fixture.history);
      assert(row, fixture.id);
      assert(
        row[references].split(";").includes(`compiler/current-api/${fixture.id}@${cohortPath}`),
      );
    }
  const reviewed = "fixture-linked;bounded-semantic-scope-reviewed;whole-fix-review-pending";
  for (const [commit, row] of indexed) {
    if (row[status] === "current-api-pinned") assert(pinned.has(commit), commit);
    else if (pinned.has(commit)) assert.equal(row[status], reviewed, commit);
  }
  const dispositions = readDispositions(root);
  for (const [commit, state] of dispositions) assert.equal(indexed.get(commit)?.[status], state);
  assert.equal(
    rows.filter((line) => line.split("\t")[status] === "no-compile-behavior").length,
    dispositions.size,
  );

  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  assert.deepEqual(receipt.currentApiCohort, {
    path: COHORT,
    sha256: digest(fs.readFileSync(path.join(root, COHORT), "utf8")),
  });
  assert.deepEqual(receipt.jsxApiCohort, {
    path: JSX_COHORT,
    sha256: digest(fs.readFileSync(path.join(root, JSX_COHORT), "utf8")),
  });
  assert.deepEqual(receipt.dispositions, {
    path: DISPOSITIONS,
    sha256: digest(fs.readFileSync(path.join(root, DISPOSITIONS), "utf8")),
  });
  assert.equal(
    receipt.currentApiPinnedFixSubjects,
    rows.filter((line) => line.split("\t")[status] === "current-api-pinned").length,
  );
  assert.equal(receipt.noCompileBehaviorFixSubjects, dispositions.size);
  const runner = fs.readFileSync(
    path.join(root, "crates/vize_atelier_sfc/tests/compile_history.rs"),
    "utf8",
  );
  assert.equal(Number(/assert_eq!\(\s*cases\.len\(\),\s*(\d+),/.exec(runner)?.[1]), cohort.length);
  const jsxRunner = fs.readFileSync(
    path.join(root, "crates/vize_atelier_jsx/tests/jsx_history.rs"),
    "utf8",
  );
  assert.equal(
    Number(/assert_eq!\(\s*cases\.len\(\),\s*(\d+),/.exec(jsxRunner)?.[1]),
    jsxCohort.length,
  );
});
