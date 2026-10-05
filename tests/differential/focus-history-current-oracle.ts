import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { compareBytes } from "./compare.mjs";
import { sha256 } from "./harness.mjs";
import { loadFocusCases, rawBytes } from "./focus-history.ts";
import { validateFocusCapture } from "./focus-history-report.ts";

export const FOCUS_CURRENT_CAPTURE_PATH =
  "crates/vize_patina/tests/fixtures/focus-history/reviewed-current-capture.capture";
export const FOCUS_CURRENT_CAPTURE_SHA256 =
  "736810bac683e88f986dd1c10f6cd0920a161ec47d868b837ef9a0477003c9e4";
export const FOCUS_CURRENT_QUALIFICATION = {
  authority: "reviewed-complete-current-output/original-count-only-witness",
  sourceRevision: "7f7b63122456fd066c86bcab7c281c9c6c9d389a",
  sourceTree: "44eeeedb7ed5198f04392f19b84d0ded92185cc1",
  workflowRun: 37170632227,
  artifactId: 11291905348,
  artifactName: "formatter-api-corpus-37170632227-1-pr-tooling-scripts-3",
  archiveSha256: "f06576ad05fef31a95b67f8dbb1c2c0571ad467009c1481282dbd5b8e59364da",
  buildReceiptSha256: "ab4e3d96aacafcb604ca4536970151be6c5e1aac98abe1839e317ffbf3e85416",
};

/** Read the unchanged, independently authenticated first hosted packet.
 * Its original unreviewed metadata and all 32 raw attempts remain historical.
 * The reviewed legacy outputs and original current native refusals remain a
 * single immutable authority; no native-equivalence decision follows.
 */
export function loadFocusCurrentOracle(
  root: string,
  bytes = fs.readFileSync(path.join(root, FOCUS_CURRENT_CAPTURE_PATH)),
) {
  assert.equal(sha256(bytes), FOCUS_CURRENT_CAPTURE_SHA256, "reviewed whole packet drift");
  const oracle = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
  validateFocusCapture(loadFocusCases(root), oracle, oracle.buildReceipt);
  assert.equal(oracle.sourceRevision, FOCUS_CURRENT_QUALIFICATION.sourceRevision);
  assert.equal(oracle.buildReceipt.source.sourceTree, FOCUS_CURRENT_QUALIFICATION.sourceTree);
  assert.equal(oracle.summary.legacyCaptured, 8);
  assert.equal(oracle.summary.nativeRefused, 8);
  assert.equal(oracle.summary.captureFailures, 0);
  return oracle;
}

function compareLane(lane: any, expected: any, admittedState: string) {
  const expectedBytes = rawBytes(expected.attempts[0].stdoutBase64);
  const wholeComparisons = lane.attempts.map((attempt: any) =>
    compareBytes(expectedBytes, rawBytes(attempt.stdoutBase64)),
  );
  return {
    state:
      lane.state !== admittedState
        ? "failed"
        : wholeComparisons.every((comparison: any) => comparison.state === "equal")
          ? "equal-reviewed-complete-current-output"
          : "current-output-drift",
    wholeComparisons,
  };
}

/** Retain every complete comparison before aggregate rejection, including
 * failed streams and full native refusal context. Native outputs are compared
 * only to their current refusal baseline, never to legacy product outputs.
 */
export function compareFocusCurrentOutputs(root: string, report: any, receipt: any) {
  const fixtures = loadFocusCases(root);
  validateFocusCapture(fixtures, report, receipt);
  const oracle = loadFocusCurrentOracle(root);
  const rows = report.rows.map((row: any, index: number) => {
    const expected = oracle.rows[index];
    return {
      id: row.id,
      inputSha256: row.inputSha256,
      sourceSha256: row.sourceSha256,
      legacy: compareLane(row.legacy, expected.legacy, "captured"),
      native: compareLane(row.native, expected.native, "refused"),
      comparison: { state: "not-compared", reason: "unprovided-rule" },
    };
  });
  return {
    schema: "vize.focus-history.current-output-comparison",
    version: 2,
    qualification: FOCUS_CURRENT_QUALIFICATION,
    oraclePacketSha256: FOCUS_CURRENT_CAPTURE_SHA256,
    sourceRevision: report.sourceRevision,
    buildReceipt: receipt,
    capture: report,
    rows,
    summary: {
      reviewedCurrentOutputOracles: 8,
      completeLegacyMatches: rows.filter(
        (row: any) => row.legacy.state === "equal-reviewed-complete-current-output",
      ).length,
      nativeRefusalMatches: rows.filter(
        (row: any) => row.native.state === "equal-reviewed-complete-current-output",
      ).length,
      nativeRefused: report.summary.nativeRefused,
      currentOutputDrift: rows
        .flatMap((row: any) => [row.legacy, row.native])
        .filter((lane: any) => lane.state === "current-output-drift").length,
      captureFailures: rows
        .flatMap((row: any) => [row.legacy, row.native])
        .filter((lane: any) => lane.state === "failed").length,
      wholeCurrentComparisons: rows
        .flatMap((row: any) => [row.legacy, row.native])
        .reduce((count: number, lane: any) => count + lane.wholeComparisons.length, 0),
      historicalCompleteOutputAuthorities: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
}

/** A serialized comparison cannot invent clean rows, provenance or credit. */
export function validateFocusCurrentComparison(root: string, comparison: any, receipt: any) {
  assert.deepEqual(comparison, compareFocusCurrentOutputs(root, comparison.capture, receipt));
  return comparison.summary;
}
