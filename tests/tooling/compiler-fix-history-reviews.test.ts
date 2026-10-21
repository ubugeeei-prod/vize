import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { validateHistoryReviews } from "../../tools/commands/fixtures/compiler-fix-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const indexPath = path.join(root, "docs/davinci/plan/compiler-fix-history.tsv");
const receiptPath = path.join(root, "docs/davinci/plan/compiler-fix-history-index.json");
const digest = (bytes: string) => createHash("sha256").update(bytes).digest("hex");

void test("bounded compiler history reviews stay tied to complete differential fixtures", () => {
  const reviewed = validateHistoryReviews(root);
  assert.equal(reviewed.reviews.size, 12);

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
  assert.equal(rows.filter((line) => line.split("\t")[status] === reviewedStatus).length, 12);

  const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8"));
  assert.equal(receipt.version, 3);
  assert.equal(receipt.nonMergeFixSubjects, rows.length);
  assert.equal(receipt.boundedReviewedFixtureScopes, 12);
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
